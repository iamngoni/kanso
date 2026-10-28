//! The preview in the app: the page, what a tap on it is, and the writes a
//! tap makes.

mod common;

use common::*;
use kanso_core::DocType;
use kanso_mobile::{
    DiagramRenderer, KansoError, PaletteDto, PreviewLink, PreviewOptions, Renderer, Ticked,
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
    // Locked and closed: the link can't be read, so the file isn't given.
    library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    library.locks().lock().unwrap();
    assert_eq!(library.attachment(&file), Err(KansoError::NotFound));
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
