//! A canvas embedded in a note, in the phone's preview (#45): drawn from the
//! canvas note's body, and never drawn while the canvas is locked.

mod common;

use common::*;
use kanso_mobile::{PaletteDto, PreviewBlock, PreviewCanvasShown, PreviewOptions, Renderer};

fn palette() -> PaletteDto {
    let theme = r##"{"name":"t","appearance":"light","ui":{"editor":"#ffffff","text":"#222222","accent":"#2185d0"},"syntax":{"keyword":"#aa0000"},"preview":{}}"##;
    Renderer::theme_tokens(theme.into()).unwrap().palette
}

fn shown(blocks: Vec<PreviewBlock>) -> PreviewCanvasShown {
    blocks
        .into_iter()
        .find_map(|block| match block {
            PreviewBlock::Canvas { shown, .. } => Some(shown),
            _ => None,
        })
        .expect("a canvas block")
}

#[test]
fn a_canvas_embed_draws_and_a_locked_one_never_does() {
    let phone = lone_phone();
    let library = &phone.library;
    let canvas = library
        .create_canvas(None, None, "Roadmap".to_owned())
        .unwrap()
        .id;
    let session = library.canvas(&canvas).unwrap();
    session
        .add_text_card(0.0, 0.0, "Launch plan".to_owned(), vec![])
        .unwrap();
    session.save().unwrap();
    let fence = format!("```canvas\n{}\n```\n", canvas);
    let host = note(library, "Host", &fence);
    let renderer = library.renderer();

    let drawn = shown(
        renderer
            .preview_blocks(&host, palette(), PreviewOptions::default())
            .unwrap(),
    );
    let PreviewCanvasShown::Drawing { svg, .. } = drawn else {
        panic!("not drawn: {drawn:?}");
    };
    assert!(svg.markup.contains("Launch plan"));

    library
        .locks()
        .lock_note(&canvas, "lock password".into())
        .unwrap();
    let locked = shown(
        renderer
            .preview_blocks(&host, palette(), PreviewOptions::default())
            .unwrap(),
    );
    assert_eq!(locked, PreviewCanvasShown::Locked);
    let page = renderer.preview_html(&host, palette()).unwrap();
    assert!(!page.contains("Launch plan"));
}

/// The phone opens an embedded canvas with the link the widget makes from
/// the block's `note:` ID (`canvasLink` in preview_figure.dart): the bare
/// ID after `inkdrop://note/`, which names the canvas. With `note:` left
/// in, the link names no note and a tap does nothing.
#[test]
fn the_embed_tap_link_opens_the_canvas() {
    use kanso_mobile::PreviewLink;
    let phone = lone_phone();
    let library = &phone.library;
    let canvas = library
        .create_canvas(None, None, "Roadmap".to_owned())
        .unwrap()
        .id;
    let host = note(library, "Host", &format!("```canvas\n{canvas}\n```\n"));
    let renderer = library.renderer();
    let bare = canvas.strip_prefix("note:").unwrap();
    let opened = renderer
        .link_target(&host, &format!("inkdrop://note/{bare}"))
        .unwrap();
    assert!(matches!(opened, PreviewLink::Note { ref id, .. } if *id == canvas));
    let doubled = renderer
        .link_target(&host, &format!("inkdrop://note/{canvas}"))
        .unwrap();
    assert_eq!(doubled, PreviewLink::Nothing);
}
