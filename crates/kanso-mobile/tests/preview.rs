//! The preview in the app: the page, what a tap on it is, and the writes a
//! tap makes.

mod common;

use common::*;
use kanso_core::DocType;
use kanso_mobile::{
    DiagramRenderer, KansoError, PaletteDto, PreviewBlock, PreviewInline, PreviewLink,
    PreviewOptions, Range16, Renderer, Ticked,
};
use kanso_store::{Origin, Put};
use proptest::prelude::*;
use serde_json::json;

fn palette() -> PaletteDto {
    let theme = r##"{"name":"t","appearance":"light","ui":{"editor":"#ffffff","text":"#222222","accent":"#2185d0"},"syntax":{"keyword":"#aa0000"},"preview":{}}"##;
    Renderer::theme_tokens(theme.into()).unwrap().palette
}

fn keyword_styles() -> Vec<kanso_mobile::SyntaxStyleRow> {
    Renderer::theme_tokens(
        r##"{"name":"t","appearance":"light","ui":{"editor":"#ffffff","text":"#222222","accent":"#2185d0"},"syntax":{"keyword":"#aa0000"},"preview":{}}"##.into(),
    )
    .unwrap()
    .syntax
}

fn bare(id: &str) -> &str {
    id.trim_start_matches("note:")
}

#[test]
fn the_page_keeps_the_addresses_the_app_reads() {
    let phone = lone_phone();
    let library = &phone.library;
    let other = note(library, "Other", "# Plan\n\ntext\n");
    let body = "[[Other#Plan]] and [[Missing]] #work/clients\n\n- [ ] one\n- [x] two @2020-01-01\n\n```rust\nfn main() {}\n```\n\n[web](https://example.com) [bad](javascript:alert(1))\n";
    let id = note(library, "Home", body);
    let page = library.renderer().preview_html(&id, palette()).unwrap();
    assert!(
        page.contains(&format!("inkdrop://note/{}#Plan", bare(&other))),
        "{page}"
    );
    assert!(page.contains("kanso-new:Missing"), "{page}");
    assert!(
        page.contains("kanso-tag:work%2Fclients") || page.contains("kanso-tag:work"),
        "{page}"
    );
    assert!(
        page.contains("kanso-task:0") && page.contains("kanso-task:1"),
        "{page}"
    );
    assert!(page.contains("kanso-copy:0"), "{page}");
    assert!(page.contains("kanso-due"), "{page}");
    assert!(page.contains("https://example.com"));
    assert!(!page.contains("javascript:"));
    assert!(!page.contains("<script"));
}

#[test]
fn a_query_fence_is_a_table_of_notes() {
    let phone = lone_phone();
    let library = &phone.library;
    let listed = note(library, "Listed", "---\nowner: ada\n---\nhello\n");
    let id = note(
        library,
        "Home",
        "```query\nfrom: hello\ncolumns: owner\n```\n\n```query\nnonsense:::\n```\n",
    );
    let page = library.renderer().preview_html(&id, palette()).unwrap();
    assert!(
        page.contains(&format!("inkdrop://note/{}", bare(&listed))),
        "{page}"
    );
    assert!(page.contains(">Listed<"));
    assert!(page.contains("kanso-query"));
}

#[test]
fn code_is_colored_with_the_given_styles_and_plain_without() {
    let phone = lone_phone();
    let id = note(
        &phone.library,
        "Code",
        "```rust\nfn main() {}\n```\n\n```nolang\nfn x\n```\n",
    );
    let renderer = phone.library.renderer();
    let plain = renderer.preview_html(&id, palette()).unwrap();
    assert!(!plain.contains("#aa0000"), "{plain}");
    let colored = renderer
        .preview_html_with(
            &id,
            palette(),
            PreviewOptions {
                code: keyword_styles(),
                ..PreviewOptions::default()
            },
        )
        .unwrap();
    assert!(colored.contains("color:#aa0000"), "{colored}");
}

#[test]
fn every_address_is_read() {
    let phone = lone_phone();
    let library = &phone.library;
    let other = note(library, "Other", "# Plan\n\ntext\n");
    let id = note(library, "Home", "x");
    let r = library.renderer();
    let t = |url: &str| r.link_target(&id, url).unwrap();
    assert_eq!(t("#plan"), PreviewLink::Heading { id: "plan".into() });
    assert_eq!(t("kanso-task:3"), PreviewLink::Task { index: 3 });
    assert_eq!(t("kanso-task:x"), PreviewLink::Nothing);
    assert_eq!(t("kanso-task:-1"), PreviewLink::Nothing);
    assert_eq!(t("kanso-task:"), PreviewLink::Nothing);
    assert_eq!(t("kanso-copy:0"), PreviewLink::CopyCode { index: 0 });
    assert_eq!(t("kanso-due:2026-01-01"), PreviewLink::Due);
    assert_eq!(
        t("kanso-tag:a%2Fb"),
        PreviewLink::Tag { name: "a/b".into() }
    );
    assert_eq!(
        t("kanso-new:Hello%20there"),
        PreviewLink::NewNote {
            title: "Hello there".into()
        }
    );
    assert_eq!(t("kanso-new:"), PreviewLink::Nothing);
    assert_eq!(
        t(&format!("inkdrop://note/{}#Plan", bare(&other))),
        PreviewLink::Note {
            id: other.clone(),
            heading: Some("Plan".into()),
            anchor: Some("plan".into())
        }
    );
    assert_eq!(
        t(&format!("inkdrop://note/{}#Nope", bare(&other))),
        PreviewLink::Note {
            id: other.clone(),
            heading: Some("Nope".into()),
            anchor: None
        }
    );
    assert_eq!(t("inkdrop://note/doesnotexist"), PreviewLink::Nothing);
    assert_eq!(
        t("https://example.com/a"),
        PreviewLink::Web {
            url: "https://example.com/a".into()
        }
    );
    assert_eq!(
        t("HTTP://EXAMPLE.COM"),
        PreviewLink::Web {
            url: "HTTP://EXAMPLE.COM".into()
        }
    );
    assert_eq!(
        t("mailto:a@b.c"),
        PreviewLink::Web {
            url: "mailto:a@b.c".into()
        }
    );
    for bad in [
        "javascript:alert(1)",
        "data:text/html,x",
        "file:///etc/passwd",
        " https://x.y",
        "https://x y",
        "tel:1",
        "",
        "inkdrop://file:",
    ] {
        assert_eq!(t(bad), PreviewLink::Nothing, "{bad:?}");
    }
    assert!(matches!(
        r.link_target("note:nope", "#a"),
        Err(KansoError::NotFound)
    ));
}

#[test]
fn a_trashed_note_is_not_a_target() {
    let phone = lone_phone();
    let library = &phone.library;
    let gone = note(library, "Gone", "x");
    let id = note(library, "Home", "x");
    library.trash(&gone).unwrap();
    assert_eq!(
        library
            .renderer()
            .link_target(&id, &format!("inkdrop://note/{}", bare(&gone)))
            .unwrap(),
        PreviewLink::Nothing
    );
}

#[test]
fn code_blocks_are_copied_by_place() {
    let phone = lone_phone();
    let id = note(
        &phone.library,
        "Code",
        "```\none\n```\n\ntext\n\n```rust\ntwo\n```\n",
    );
    let r = phone.library.renderer();
    assert_eq!(r.code_block(&id, 0).unwrap(), "one");
    assert_eq!(r.code_block(&id, 1).unwrap(), "two");
    assert_eq!(r.code_block(&id, 2), Err(KansoError::NotFound));
}

#[test]
fn ticking_a_task_by_place() {
    let phone = lone_phone();
    let library = &phone.library;
    let id = note(
        library,
        "Tasks",
        "- [ ] a\n- [x] b\n```\n- [ ] not\n```\n- [ ] c\n",
    );
    let rev = library.note(&id).unwrap().rev;
    let Ticked::Changed { rev: new, undo, .. } = library.toggle_task_at(&id, 2, &rev).unwrap()
    else {
        panic!("not changed");
    };
    assert_eq!(
        library.note(&id).unwrap().body.unwrap(),
        "- [ ] a\n- [x] b\n```\n- [ ] not\n```\n- [x] c\n"
    );
    // A page drawn from the old revision can't tick.
    assert_eq!(
        library.toggle_task_at(&id, 0, &rev),
        Err(KansoError::Conflict {
            current_rev: new.clone()
        })
    );
    assert_eq!(
        library.toggle_task_at(&id, 9, &new).unwrap(),
        Ticked::Missing
    );
    assert_eq!(library.undo(undo).unwrap(), 1);
    assert!(
        library
            .note(&id)
            .unwrap()
            .body
            .unwrap()
            .ends_with("- [ ] c\n")
    );
}

/// Every address a block carries: links, due chips, query rows, task boxes
/// (`kanso-task:`) and copy buttons (`kanso-copy:`).
fn block_addresses(blocks: &[PreviewBlock], found: &mut Vec<String>) {
    fn inlines(list: &[PreviewInline], found: &mut Vec<String>) {
        for inline in list {
            match inline {
                PreviewInline::Link {
                    url,
                    inlines: inner,
                } => {
                    found.push(url.clone());
                    inlines(inner, found);
                }
                PreviewInline::Due { url, .. } => found.push(url.clone()),
                _ => {}
            }
        }
    }
    for block in blocks {
        match block {
            PreviewBlock::Heading { inlines: list, .. }
            | PreviewBlock::Paragraph { inlines: list } => inlines(list, found),
            PreviewBlock::List { items, .. } => {
                for item in items {
                    if let Some(index) = item.task.and_then(|task| task.index) {
                        found.push(format!("kanso-task:{index}"));
                    }
                    block_addresses(&item.blocks, found);
                }
            }
            PreviewBlock::Quote { blocks } => block_addresses(blocks, found),
            PreviewBlock::Code { copy, .. } => found.push(format!("kanso-copy:{copy}")),
            PreviewBlock::Failed {
                copy: Some(copy), ..
            } => found.push(format!("kanso-copy:{copy}")),
            PreviewBlock::Query { rows, .. } => {
                found.extend(rows.iter().map(|row| row.url.clone()))
            }
            _ => {}
        }
    }
}

#[test]
fn the_blocks_keep_the_pages_addresses() {
    let phone = lone_phone();
    let library = &phone.library;
    let other = note(library, "Other", "# Plan\n\ntext\n");
    let body = "# Home\n\n[[Other#Plan]] and [[Missing]] #work/clients [up](#home)\n\n- [ ] one\n- [x] two @2020-01-01\n\n```rust\nfn main() {}\n```\n\n```query\nfrom: all\n```\n\n[web](https://example.com) [bad](javascript:alert(1))\n";
    let id = note(library, "Home", body);
    let renderer = library.renderer();
    let options = PreviewOptions {
        code: keyword_styles(),
        ..PreviewOptions::default()
    };
    let page = renderer
        .preview_html_with(&id, palette(), options.clone())
        .unwrap();
    let blocks = renderer.preview_blocks(&id, palette(), options).unwrap();
    let mut found = Vec::new();
    block_addresses(&blocks, &mut found);
    assert!(found.contains(&format!("inkdrop://note/{}#Plan", bare(&other))));
    assert!(found.contains(&"#home".to_owned()), "{found:?}");
    assert!(found.contains(&"kanso-task:0".to_owned()));
    assert!(found.contains(&"kanso-task:1".to_owned()));
    assert!(found.contains(&"kanso-copy:0".to_owned()));
    assert!(found.iter().any(|url| url.starts_with("kanso-due:")));
    assert!(!found.iter().any(|url| url.contains("javascript")));
    // Each address is one the page carries, and each reads as a target.
    for url in &found {
        assert!(
            page.contains(&format!("href=\"{url}\"")),
            "{url} not in {page}"
        );
        assert_ne!(
            renderer.link_target(&id, url).unwrap(),
            PreviewLink::Nothing,
            "{url}"
        );
    }
    // Code is colored as the page colors it.
    assert!(blocks.iter().any(|block| matches!(
        block,
        PreviewBlock::Code { runs, .. } if runs.iter().any(|run| run.color == Some(0xaa0000))
    )));
}

#[test]
fn a_locked_note_has_no_blocks_while_closed() {
    let phone = lone_phone();
    let library = &phone.library;
    let id = note(library, "Secret", "# Hidden\n");
    library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    assert!(
        library
            .renderer()
            .preview_blocks(&id, palette(), PreviewOptions::default())
            .is_ok()
    );
    library.locks().lock().unwrap();
    assert_eq!(
        library
            .renderer()
            .preview_blocks(&id, palette(), PreviewOptions::default()),
        Err(KansoError::Locked)
    );
}

#[test]
fn a_locked_note_ticks_only_while_open() {
    let phone = lone_phone();
    let library = &phone.library;
    let id = note(library, "Tasks", "- [ ] secret\n");
    library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    let rev = library.note(&id).unwrap().rev;
    assert!(matches!(
        library.toggle_task_at(&id, 0, &rev).unwrap(),
        Ticked::Changed { .. }
    ));
    library.locks().lock().unwrap();
    let rev = library.note(&id).unwrap().rev;
    assert_eq!(
        library.toggle_task_at(&id, 0, &rev),
        Err(KansoError::Locked)
    );
    assert_eq!(
        library.renderer().code_block(&id, 0),
        Err(KansoError::Locked)
    );
}

#[test]
fn a_linked_note_is_made_once_beside_the_one_that_links() {
    let phone = lone_phone();
    let library = &phone.library;
    let book = library.create_notebook("Garden".into(), None).unwrap();
    let from = library
        .create(kanso_mobile::NewNote {
            notebook_id: Some(book.id.clone()),
            title: "From".into(),
            body: "[[Seed]]".into(),
            ..Default::default()
        })
        .unwrap();
    let made = library.create_linked_note("  Seed  ", &from.id).unwrap();
    let view = library.note(&made.id).unwrap();
    assert_eq!(view.title, "Seed");
    assert_eq!(view.notebook_id, book.id);
    assert_eq!(
        library.create_linked_note("seed", &from.id).unwrap().id,
        made.id
    );
    assert!(matches!(
        library.create_linked_note("   ", &from.id),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(
        library.create_linked_note("X", "note:nope"),
        Err(KansoError::NotFound)
    );
}

#[test]
fn an_attachment_comes_back_only_while_a_readable_note_links_it() {
    let phone = lone_phone();
    let library = &phone.library;
    let store = kanso_store::Store::open(phone.path());
    drop(store);
    let file = library_store_put(&phone);
    assert_eq!(
        library.attachment(&file),
        Err(KansoError::NotFound),
        "nothing links it"
    );
    let id = note(
        library,
        "Doc",
        &format!(
            "[a file](inkdrop://file:{})",
            file.trim_start_matches("file:")
        ),
    );
    let got = library.attachment(&file).unwrap();
    assert_eq!(
        (
            got.name.as_str(),
            got.content_type.as_str(),
            got.bytes.as_slice()
        ),
        ("hello.png", "image/png", &b"hello"[..])
    );
    assert_eq!(
        library
            .renderer()
            .link_target(
                &id,
                &format!("inkdrop://file:{}", file.trim_start_matches("file:"))
            )
            .unwrap(),
        PreviewLink::Attachment {
            id: file.clone(),
            name: "hello.png".into(),
            content_type: "image/png".into(),
            size: 5
        }
    );
    // Locked and closed: the link can't be read, so the sealed file says it
    // is locked rather than pretending it is missing.
    library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    library.locks().lock().unwrap();
    assert_eq!(library.attachment(&file), Err(KansoError::Locked));
    library.locks().unlock(LOCK_PASSWORD.into()).unwrap();
    assert!(library.attachment(&file).is_ok());
    assert_eq!(library.attachment("note:x"), Err(KansoError::NotFound));
}

/// A file made through the library's own store.
fn library_store_put(phone: &Phone) -> String {
    // The library holds the database; write through a second handle.
    let store = kanso_store::Store::open(phone.path()).unwrap();
    store
        .put(
            DocType::File,
            json!({"name": "hello.png", "contentType": "image/png", "_attachments": {"index": {"content_type": "image/png", "data": "aGVsbG8="}}}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

#[test]
fn the_diagram_renderer_is_set_and_read() {
    assert_eq!(Renderer::diagram_renderer(), DiagramRenderer::Pele);
    Renderer::set_diagram_renderer(DiagramRenderer::Kanso);
    assert_eq!(Renderer::diagram_renderer(), DiagramRenderer::Kanso);
    Renderer::set_diagram_renderer(DiagramRenderer::Pele);
    assert_eq!(Renderer::diagram_renderer(), DiagramRenderer::Pele);
}

// MARK: The blocks of an editor's text

#[test]
fn an_editors_text_is_drawn_with_its_lines() {
    let phone = lone_phone();
    let library = &phone.library;
    note(library, "Other", "x");
    let body = "# Home\n\n[[Other]] and [[Missing]] #tag @2020-01-01\n\n- [ ] one\n- [x] two\n\n```rust\nfn main() {}\n```\n\nCite[^1] é🙂\n\n[^1]: note\n";
    let id = note(library, "Home", body);
    let renderer = library.renderer();
    let saved = renderer
        .preview_blocks(&id, palette(), PreviewOptions::default())
        .unwrap();
    let sourced = renderer
        .preview_blocks_for_text(&id, body, palette(), PreviewOptions::default())
        .unwrap();
    // The same blocks as the saved note's, addresses and all.
    let blocks: Vec<PreviewBlock> = sourced.iter().map(|b| b.block.clone()).collect();
    assert_eq!(blocks, saved);
    let lines: Vec<_> = sourced.iter().map(|b| (b.first_line, b.end_line)).collect();
    assert_eq!(
        lines,
        vec![
            (Some(0), Some(1)),
            (Some(2), Some(3)),
            (Some(4), Some(6)),
            (Some(7), Some(10)),
            (Some(11), Some(12)),
            // The footnotes after the note come from nowhere.
            (None, None),
            (None, None),
        ]
    );
    let units = |s: &str| s.encode_utf16().count() as u32;
    let fence = units(&body[..body.find("```rust").unwrap()]);
    assert_eq!(
        sourced[3].lines,
        Some(Range16 {
            start: fence,
            end: fence + units("```rust\nfn main() {}\n```"),
        })
    );
    // A list's whole lines, without the line break after its last.
    let list = units(&body[..body.find("- [ ]").unwrap()]);
    assert_eq!(
        sourced[2].lines,
        Some(Range16 {
            start: list,
            end: list + units("- [ ] one\n- [x] two"),
        })
    );

    // Text not saved yet is drawn as it is, for a locked note too.
    library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    library.locks().lock().unwrap();
    let unsaved = renderer
        .preview_blocks_for_text(
            &id,
            "$$\nx\n$$\n\n| a |\n| - |\n",
            palette(),
            PreviewOptions::default(),
        )
        .unwrap();
    assert!(matches!(unsaved[0].block, PreviewBlock::Figure { .. }));
    assert_eq!(
        (unsaved[0].first_line, unsaved[0].end_line),
        (Some(0), Some(3))
    );
    assert!(matches!(unsaved[1].block, PreviewBlock::Table { .. }));
    assert_eq!(
        (unsaved[1].first_line, unsaved[1].end_line),
        (Some(4), Some(6))
    );
    let empty = renderer
        .preview_blocks_for_text(&id, "", palette(), PreviewOptions::default())
        .unwrap();
    assert!(empty.is_empty());
}

fn piece() -> impl Strategy<Value = &'static str> {
    prop::sample::select(vec![
        "# Head é [[Other]]\n",
        "para **b** #tag @2020-01-01 [[Gone]] 🙂\n",
        "- a\n  - b\n",
        "- [ ] t\n- [x] d\n",
        "> q\n",
        "```js\nlet a;\n```\n",
        "| a | b |\n| - | - |\n| 1 | 2 |\n",
        "$$\nx^2\n$$\n",
        "<div>x</div>\n",
        "text\n",
        "\n",
    ])
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// For any text, the blocks are the saved note's, their lines in order
    /// and inside the text, and each top-level widget Live mode draws has
    /// the block drawn from its lines.
    #[test]
    fn prop_editor_blocks_match_their_lines(pieces in prop::collection::vec(piece(), 0..8)) {
        let body: String = pieces.concat();
        let phone = lone_phone();
        let library = &phone.library;
        note(library, "Other", "x");
        let id = note(library, "N", &body);
        let renderer = library.renderer();
        let saved = renderer.preview_blocks(&id, palette(), PreviewOptions::default()).unwrap();
        let sourced = renderer
            .preview_blocks_for_text(&id, &body, palette(), PreviewOptions::default())
            .unwrap();
        prop_assert_eq!(sourced.iter().map(|b| b.block.clone()).collect::<Vec<_>>(), saved);
        let total = body.encode_utf16().count() as u32;
        let count = body.split('\n').count() as u32;
        let mut last = 0;
        for block in &sourced {
            let (Some(first), Some(end), Some(lines)) = (block.first_line, block.end_line, block.lines) else {
                continue;
            };
            prop_assert!(first >= last && end > first && end <= count);
            last = first;
            prop_assert!(lines.start <= lines.end && lines.end <= total);
        }
        let session = library.edit(&id).unwrap();
        prop_assert_eq!(session.snapshot().unwrap().text, body.clone());
        let doc = session.live_document(Vec::new()).unwrap();
        for widget in &doc.widgets {
            let top_level = doc.blocks.iter().any(|block| block.first_line == widget.first_line
                && block.list_depth == 0 && block.quote_depth == 0);
            if !top_level {
                continue;
            }
            prop_assert!(
                sourced.iter().any(|block| block.first_line == Some(widget.first_line)
                    && block.end_line == Some(widget.end_line)
                    && block.lines == Some(widget.lines)),
                "{:?} in {:?}", widget, body
            );
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Whatever the text of an address, reading it never panics, and only a
    /// web or mail address is ever `Web`, kept as written.
    #[test]
    fn prop_an_address_is_read_safely(url in "\\PC{0,40}") {
        let phone = lone_phone();
        let id = note(&phone.library, "Home", "x");
        let target = phone.library.renderer().link_target(&id, &url).unwrap();
        if let PreviewLink::Web { url: web } = &target {
            prop_assert_eq!(web, &url);
            let lower = url.to_ascii_lowercase();
            prop_assert!(lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("mailto:"));
            prop_assert!(!url.chars().any(|c| c.is_whitespace() || c.is_control()));
        }
    }

    /// Any note text makes a page with nothing active in it, and the n-th
    /// task link is the n-th task: ticking it flips exactly that task.
    #[test]
    fn prop_the_page_is_inert_and_boxes_are_the_tasks(
        lines in proptest::collection::vec(prop_oneof![
            Just("- [ ] a".to_owned()), Just("- [x] b".to_owned()), Just("text".to_owned()),
            Just("```".to_owned()), Just("<script>x()</script>".to_owned()),
            Just("[x](javascript:1)".to_owned()), Just("> - [ ] q".to_owned()),
        ], 0..10),
        pick in 0usize..6,
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        let body = lines.join("\n");
        let id = note(library, "N", &body);
        let page = library.renderer().preview_html(&id, palette()).unwrap();
        prop_assert!(!page.contains("<script>x"), "{page}");
        prop_assert!(!page.contains("href=\"javascript") && !page.contains("src=\"javascript"));
        let tasks = kanso_core::tasks::task_items(&body).len();
        let rev = library.note(&id).unwrap().rev;
        match library.toggle_task_at(&id, pick as u32, &rev).unwrap() {
            Ticked::Missing => prop_assert!(pick >= tasks),
            Ticked::Changed { .. } => {
                prop_assert!(pick < tasks);
                let after = kanso_core::tasks::task_items(&library.note(&id).unwrap().body.unwrap());
                let before = kanso_core::tasks::task_items(&body);
                for (n, (b, a)) in before.iter().zip(&after).enumerate() {
                    prop_assert_eq!(a.checked, if n == pick { !b.checked } else { b.checked });
                }
            }
            Ticked::Unchanged => prop_assert!(false),
        }
    }
}

/// A recording's transcript (#38): the markers draw as nothing, the stamps
/// draw as text, and a tap on one is inert until the phone has a player.
#[test]
fn a_recording_transcript_draws_and_its_moment_links_are_inert() {
    let phone = lone_phone();
    let body = "Before\n\n<!-- kanso:transcript -->\n[00:00](kanso-audio://file:m1?t=0) We started.\n\n[00:20](kanso-audio://file:m1?t=20) Then this.\n<!-- /kanso:transcript -->\n\nAfter\n";
    let id = note(&phone.library, "Meeting", body);
    let r = phone.library.renderer();
    let blocks = format!(
        "{:?}",
        r.preview_blocks(&id, palette(), PreviewOptions::default())
            .unwrap()
    );
    assert!(
        blocks.contains("00:20") && blocks.contains("Then this."),
        "{blocks}"
    );
    assert!(!blocks.contains("kanso:transcript"), "{blocks}");
    let page = r.preview_html(&id, palette()).unwrap();
    assert!(page.contains("Then this."), "{page}");
    for url in ["kanso-audio://file:m1?t=20", "kanso-audio://file:m1"] {
        assert_eq!(r.link_target(&id, url).unwrap(), PreviewLink::Nothing);
    }
}
