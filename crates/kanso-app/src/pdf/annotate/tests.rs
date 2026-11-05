//! Highlights in the reader, headless and without a worker (issue #43,
//! step 6): the pointer selects, a colour highlights, the quote and the
//! block go out as writes, the panel lists and changes them.
//!
//! The pages are not drawn by PDFium here; the reader is told they opened and
//! given each page's characters, so these run on every machine.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui_kit::{
    AnyWindowHandle, AppContext as _, Entity, Modifiers, MouseButton, TestAppContext,
    VisualTestContext, point,
};
use kanso_pdf::highlights::{Annotations, Status};
use kanso_pdf::model::{Color, Quad};
use kanso_pdf::{PageText, TextChar};
use kanso_store::Store;
use proptest::prelude::*;

use super::super::layout::{Layout, MARGIN, PAGE_GAP};
use super::{SAVE_DELAY, Write, page_point};
use crate::pdf::{PdfEvent, PdfReader};

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

    /// A point inside a page maps to where it is on that page, and back.
    #[test]
    fn a_point_on_a_page_maps_to_its_place_and_back(
        sizes in proptest::collection::vec((100f32..900., 100f32..900.), 1..8),
        scale in 0.4f32..3.0,
        viewport in 300f32..2000.,
        page_pick in 0usize..8,
        nx in 0.0f64..=1.0,
        ny in 0.0f64..=1.0,
    ) {
        let points: Vec<[f32; 2]> = sizes.iter().map(|(w, h)| [*w, *h]).collect();
        let layout = Layout::new(&points, scale, PAGE_GAP, MARGIN);
        let content = (layout.max_width() + 2. * MARGIN).max(viewport);
        let page = page_pick % layout.pages();
        let [w, h] = layout.size(page);
        let x = (content - w) / 2. + nx as f32 * w;
        let y = layout.top(page) + ny as f32 * h;
        let (p, gx, gy) = page_point(&layout, content, x, y, false).unwrap();
        prop_assert_eq!(p, page);
        prop_assert!((gx - nx).abs() < 1e-3, "x {} vs {}", gx, nx);
        prop_assert!((gy - ny).abs() < 1e-3, "y {} vs {}", gy, ny);
    }

    /// Off every page there is no place, unless the pointer is being dragged:
    /// then the nearest page's edge, never outside the page.
    #[test]
    fn a_dragged_pointer_stays_on_a_page(
        sizes in proptest::collection::vec((100f32..900., 100f32..900.), 1..8),
        viewport in 300f32..2000.,
        x in -500f32..3000.,
        y in -500f32..9000.,
    ) {
        let points: Vec<[f32; 2]> = sizes.iter().map(|(w, h)| [*w, *h]).collect();
        let layout = Layout::new(&points, 1., PAGE_GAP, MARGIN);
        let content = (layout.max_width() + 2. * MARGIN).max(viewport);
        let (page, gx, gy) = page_point(&layout, content, x, y, true).unwrap();
        prop_assert!(page < layout.pages());
        prop_assert!((0.0..=1.0).contains(&gx) && (0.0..=1.0).contains(&gy));
        if let Some((q, ..)) = page_point(&layout, content, x, y, false) {
            prop_assert_eq!(q, page, "on a page, the same one");
        }
    }
}

const FILE_BYTES: &[u8] = b"%PDF-1.7\n%body";

/// "Revenue rose" on the first line of a page, "twelve percent" on the
/// second: boxes 0.02 of the page wide, a word gap of one box.
fn page_text(page: u32, lines: &[&str]) -> PageText {
    let mut chars = Vec::new();
    for (row, line) in lines.iter().enumerate() {
        let y0 = 0.10 + row as f64 * 0.035;
        let mut x = 0.10;
        for word in line.split(' ') {
            for ch in word.chars() {
                chars.push(TextChar {
                    ch,
                    quad: Quad::new(x, y0, x + 0.02, y0 + 0.025),
                    turns: 0,
                });
                x += 0.02;
            }
            x += 0.02;
        }
    }
    PageText {
        page,
        width: 612.,
        height: 792.,
        rotation: 0,
        chars,
    }
}

struct Fixture {
    window: AnyWindowHandle,
    reader: Entity<PdfReader>,
    id: String,
    writes: Rc<RefCell<Vec<Write>>>,
    _dir: tempfile::TempDir,
}

fn open(cx: &mut TestAppContext, body: &str, pages: usize) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("k.db")).unwrap());
    let id = kanso_engine::attach::attach_pdf(&store, FILE_BYTES, "Report.pdf", None).unwrap();
    let (window, reader) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| {
                PdfReader::with_paths(
                    store.clone(),
                    id.clone(),
                    "Report.pdf".into(),
                    1,
                    None,
                    window,
                    cx,
                )
            })
        })
        .unwrap()
    });
    let writes = Rc::new(RefCell::new(Vec::new()));
    let sink = writes.clone();
    cx.update(|cx| {
        cx.subscribe(&reader, move |_, event: &PdfEvent, _| {
            if let PdfEvent::Write(write) = event {
                sink.borrow_mut().push(write.clone());
            }
        })
        .detach();
    });
    // The missing worker fails the open at once; then the test opens it.
    cx.run_until_parked();
    let body = body.to_owned();
    let sizes = vec![[612., 792.]; pages];
    cx.update_window(window, |_, window, cx| {
        reader.update(cx, |reader, cx| {
            reader.set_note(body, None, cx);
            reader.test_open(sizes, window, cx);
            reader.test_text(page_text(1, &["Revenue rose", "twelve percent"]), cx);
            reader.test_text(page_text(2, &["next page"]), cx);
        })
    })
    .unwrap();
    let f = Fixture {
        window,
        reader,
        id,
        writes,
        _dir: dir,
    };
    draw(cx, &f);
    f
}

fn draw(cx: &mut TestAppContext, f: &Fixture) {
    for _ in 0..3 {
        cx.run_until_parked();
        f.window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
    }
    cx.run_until_parked();
}

fn on_reader<R>(
    cx: &mut TestAppContext,
    f: &Fixture,
    what: impl FnOnce(&mut PdfReader, &mut gpui_kit::Window, &mut gpui_kit::Context<PdfReader>) -> R,
) -> R {
    let out = f
        .window
        .update(cx, |_, window, cx| {
            f.reader.update(cx, |reader, cx| what(reader, window, cx))
        })
        .unwrap();
    draw(cx, f);
    out
}

/// A window position on page 1, from its normalised place.
fn on_page(
    cx: &mut TestAppContext,
    f: &Fixture,
    nx: f32,
    ny: f32,
) -> gpui_kit::Point<gpui_kit::Pixels> {
    let mut vcx = VisualTestContext::from_window(f.window, cx);
    let b = vcx.debug_bounds("pdf-page-0").expect("page 1 is drawn");
    point(
        b.origin.x + b.size.width * nx,
        b.origin.y + b.size.height * ny,
    )
}

fn drag(cx: &mut TestAppContext, f: &Fixture, from: (f32, f32), to: (f32, f32)) {
    let (a, b) = (on_page(cx, f, from.0, from.1), on_page(cx, f, to.0, to.1));
    let mut vcx = VisualTestContext::from_window(f.window, cx);
    vcx.simulate_mouse_down(a, MouseButton::Left, Modifiers::default());
    vcx.simulate_mouse_move(b, MouseButton::Left, Modifiers::default());
    vcx.simulate_mouse_up(b, MouseButton::Left, Modifiers::default());
    draw(cx, f);
}

fn click_selector(cx: &mut TestAppContext, f: &Fixture, selector: &'static str) {
    let mut vcx = VisualTestContext::from_window(f.window, cx);
    let at = vcx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} isn't on screen"))
        .center();
    vcx.simulate_click(at, Modifiers::default());
    draw(cx, f);
}

fn shown(cx: &mut TestAppContext, f: &Fixture, selector: &'static str) -> bool {
    VisualTestContext::from_window(f.window, cx)
        .debug_bounds(selector)
        .is_some()
}

fn selected_text(cx: &mut TestAppContext, f: &Fixture) -> Option<String> {
    on_reader(cx, f, |reader, _, _| {
        let selection = reader.notes.selection?;
        Some(reader.notes.texts.text(&selection))
    })
}

/// Dragging over words selects them, shows the toolbar by them, and copies.
#[gpui_kit::test]
fn dragging_over_words_selects_them_and_the_toolbar_copies(cx: &mut TestAppContext) {
    let f = open(cx, "", 2);
    assert!(!shown(cx, &f, "pdf-toolbar"));
    drag(cx, &f, (0.105, 0.112), (0.335, 0.112));
    assert_eq!(selected_text(cx, &f).as_deref(), Some("Revenue rose"));
    assert!(shown(cx, &f, "pdf-toolbar"), "no toolbar by the selection");
    for part in [
        "pdf-color-yellow",
        "pdf-color-green",
        "pdf-color-blue",
        "pdf-color-pink",
        "pdf-add-note",
        "pdf-copy",
        "pdf-add-quote",
    ] {
        assert!(shown(cx, &f, part), "{part} is missing from the toolbar");
    }
    click_selector(cx, &f, "pdf-copy");
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("Revenue rose"));
    // A click elsewhere lets go.
    let away = on_page(cx, &f, 0.9, 0.3);
    let mut vcx = VisualTestContext::from_window(f.window, cx);
    vcx.simulate_click(away, Modifiers::default());
    draw(cx, &f);
    assert!(!shown(cx, &f, "pdf-toolbar"));
}

/// A drag across two lines is one selection, and the page break joins pages.
#[gpui_kit::test]
fn a_selection_runs_across_lines_and_pages(cx: &mut TestAppContext) {
    let f = open(cx, "", 2);
    drag(cx, &f, (0.105, 0.112), (0.21, 0.147));
    assert_eq!(
        selected_text(cx, &f).as_deref(),
        Some("Revenue rose\ntwelve")
    );
    on_reader(cx, &f, |reader, window, cx| {
        reader.clear_selection(cx);
        reader.select_all(cx);
        let id = reader.test_highlight_all(Color::Blue, window, cx).unwrap();
        let h = reader.test_notes().get(&id).unwrap().clone();
        assert_eq!(h.page, 1);
        assert_eq!(h.quads.keys().copied().collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(h.text, "Revenue rose twelve percent next page");
    });
}

/// A colour makes a highlight and, by default, writes its quote at once; a
/// burst of highlights is one save of the block, after a pause.
#[gpui_kit::test]
fn a_burst_of_highlights_is_one_save_and_each_quote_goes_at_once(cx: &mut TestAppContext) {
    let f = open(cx, "", 2);
    for (color, word) in [
        ("pdf-color-yellow", (0.105, 0.30)),
        ("pdf-color-green", (0.36, 0.60)),
        ("pdf-color-pink", (0.105, 0.30)),
    ] {
        drag(cx, &f, (word.0, 0.112), (word.1, 0.112));
        click_selector(cx, &f, color);
    }
    let quotes = f
        .writes
        .borrow()
        .iter()
        .filter(|w| matches!(w, Write::Quote { .. }))
        .count();
    let saves = f
        .writes
        .borrow()
        .iter()
        .filter(|w| matches!(w, Write::Save(_)))
        .count();
    assert_eq!(
        (quotes, saves),
        (3, 0),
        "quotes go at once, the block waits"
    );

    cx.executor()
        .advance_clock(SAVE_DELAY + std::time::Duration::from_millis(1));
    cx.run_until_parked();
    let writes = f.writes.borrow();
    let saves: Vec<_> = writes
        .iter()
        .filter_map(|w| match w {
            Write::Save(a) => Some(a),
            _ => None,
        })
        .collect();
    assert_eq!(saves.len(), 1, "a burst is one save");
    assert_eq!(saves[0].block.highlights.len(), 3);
}

/// The panel lists the highlights; a comment and a delete change them; the
/// quote can be put back.
#[gpui_kit::test]
fn the_panel_lists_comments_deletes_and_adds_quotes_back(cx: &mut TestAppContext) {
    let f = open(cx, "", 2);
    assert!(!shown(cx, &f, "pdf-panel"));
    drag(cx, &f, (0.105, 0.112), (0.335, 0.112));
    click_selector(cx, &f, "pdf-color-yellow");
    assert!(
        shown(cx, &f, "pdf-panel"),
        "the panel opens with the first highlight"
    );
    let id = on_reader(cx, &f, |r, _, _| r.highlight_ids().remove(0));
    assert!(shown(cx, &f, "pdf-panel-count"));
    // Written into the note by the editor in real use; here the note stays
    // empty, so the panel says the quote is not in it.
    assert!(
        VisualTestContext::from_window(f.window, cx)
            .debug_bounds(Box::leak(format!("pdf-not-in-note-{id}").into_boxed_str()))
            .is_some()
    );
    click_selector(
        cx,
        &f,
        Box::leak(format!("pdf-add-to-note-{id}").into_boxed_str()),
    );
    assert!(
        f.writes
            .borrow()
            .iter()
            .filter(|w| matches!(w, Write::Quote { .. }))
            .count()
            >= 2,
        "Add to note asks for the quote again"
    );

    // A comment.
    on_reader(cx, &f, |reader, window, cx| {
        reader.begin_comment(&id, window, cx);
        reader.comment.update(cx, |field, cx| {
            field.set_value("Check the board deck.", window, cx)
        });
        reader.commit_comment(window, cx);
    });
    assert_eq!(
        on_reader(cx, &f, |r, _, _| r
            .test_notes()
            .get(&id)
            .unwrap()
            .comment
            .clone())
        .as_deref(),
        Some("Check the board deck.")
    );
    assert!(
        VisualTestContext::from_window(f.window, cx)
            .debug_bounds(Box::leak(format!("pdf-comment-{id}").into_boxed_str()))
            .is_some()
    );

    // Delete.
    click_selector(
        cx,
        &f,
        Box::leak(format!("pdf-delete-{id}").into_boxed_str()),
    );
    assert!(on_reader(cx, &f, |r, _, _| r.highlight_ids()).is_empty());
    assert!(shown(cx, &f, "pdf-panel-empty"));
}

/// A block that left the note leaves the highlights on screen, read-only,
/// with Restore annotations, which asks the editor to write them back.
#[gpui_kit::test]
fn a_block_that_left_the_note_can_be_restored(cx: &mut TestAppContext) {
    let probe = open(cx, "", 2);
    let file = probe.id.clone();
    let mut a = Annotations::new(&file, "");
    let parts = {
        let t = page_text(1, &["Revenue rose"]);
        let doc = kanso_pdf::TextDoc::new([&t]);
        doc.highlight_parts(&doc.select_all().unwrap())
    };
    a.add(parts, Color::Green, 1).unwrap();
    let body = a.save_edit("Notes").unwrap().unwrap().apply("Notes");
    let f = open(cx, &body, 2);
    // The fixture's file id differs; follow the id this one has.
    let body = body.replace(&file, &f.id);
    on_reader(cx, &f, |reader, _, cx| {
        reader.note_changed(body.clone(), cx)
    });
    assert_eq!(on_reader(cx, &f, |r, _, _| r.highlight_ids().len()), 1);
    // The note is restored to a version without the block.
    on_reader(cx, &f, |reader, _, cx| {
        reader.note_changed("Notes".into(), cx)
    });
    assert_eq!(
        on_reader(cx, &f, |r, _, _| r.test_notes().status.clone()),
        Status::Restorable
    );
    assert_eq!(
        on_reader(cx, &f, |r, _, _| r.highlight_ids().len()),
        1,
        "still shown"
    );
    on_reader(cx, &f, |r, _, _| r.notes.panel = true);
    assert!(shown(cx, &f, "pdf-restore-banner"));
    // Editing is refused in words.
    on_reader(cx, &f, |reader, window, cx| {
        let made = reader.test_highlight_all(Color::Pink, window, cx);
        assert!(made.is_none(), "a read-only block took a highlight");
    });
    assert_eq!(on_reader(cx, &f, |r, _, _| r.highlight_ids().len()), 1);
    click_selector(cx, &f, "pdf-restore");
    assert!(
        f.writes
            .borrow()
            .iter()
            .any(|w| matches!(w, Write::Restore(a) if a.block.highlights.len() == 1))
    );
}

/// A link that names a highlight scrolls to it and marks it for a moment.
#[gpui_kit::test]
fn a_link_to_a_highlight_scrolls_to_it_and_flashes(cx: &mut TestAppContext) {
    let probe = open(cx, "", 6);
    let file = probe.id.clone();
    let mut a = Annotations::new(&file, "");
    let mut quads = std::collections::BTreeMap::new();
    quads.insert(5, vec![Quad::new(0.1, 0.5, 0.5, 0.52)]);
    let parts = kanso_pdf::HighlightParts {
        quads,
        text: "far away".into(),
    };
    let id = a.add(parts, Color::Yellow, 1).unwrap();
    let body = a.save_edit("").unwrap().unwrap().apply("");

    // A fresh reader, opened by a link that names the highlight.
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("k.db")).unwrap());
    let real = kanso_engine::attach::attach_pdf(&store, FILE_BYTES, "Report.pdf", None).unwrap();
    let body = body.replace(&file, &real);
    let (window, reader) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| {
                PdfReader::with_paths(
                    store.clone(),
                    real.clone(),
                    "Report.pdf".into(),
                    1,
                    None,
                    window,
                    cx,
                )
            })
        })
        .unwrap()
    });
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        reader.update(cx, |reader, cx| {
            reader.set_note(body, Some(id.clone()), cx);
            reader.test_open(vec![[612., 792.]; 6], window, cx);
        })
    })
    .unwrap();
    for _ in 0..3 {
        cx.run_until_parked();
        window
            .update(cx, |_, w, cx| {
                w.refresh();
                w.draw(cx).clear(cx);
            })
            .unwrap();
    }
    let (top, flash) = cx.read(|cx| {
        let r = reader.read(cx);
        (r.scroll_top(), r.notes.flash.clone())
    });
    assert!(top > 1000., "the reader stayed at the top: {top}");
    assert_eq!(flash.as_deref(), Some(id.as_str()));
    // And lets go of the mark a moment later.
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(3));
    cx.run_until_parked();
    assert_eq!(cx.read(|cx| reader.read(cx).notes.flash.clone()), None);
}
