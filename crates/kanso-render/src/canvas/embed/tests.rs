use std::cell::Cell;
use std::collections::HashMap;
use std::path::PathBuf;

use kanso_core::canvas::{Canvas, Node, to_body};
use proptest::prelude::*;

use super::*;

fn card(id: &str, x: f64, y: f64, w: f64, h: f64, text: &str) -> Node {
    Node {
        id: id.into(),
        kind: "text".into(),
        x,
        y,
        width: w,
        height: h,
        text: Some(text.into()),
        ..Node::default()
    }
}

fn group(id: &str, x: f64, y: f64, w: f64, h: f64) -> Node {
    Node {
        id: id.into(),
        kind: "group".into(),
        label: Some("Plan".into()),
        x,
        y,
        width: w,
        height: h,
        ..Node::default()
    }
}

fn sample() -> Canvas {
    let mut c = Canvas::default();
    c.put_node(card("n1", 0.0, 0.0, 240.0, 100.0, "Launch plan"), 1);
    c.put_node(card("n2", 400.0, 300.0, 200.0, 80.0, "Far away"), 2);
    c.put_node(group("g1", -20.0, -20.0, 300.0, 160.0), 3);
    c
}

fn spec(group: Option<&str>) -> Spec {
    Spec {
        title: "Roadmap".into(),
        group: group.map(Into::into),
        width: None,
    }
}

fn palette() -> Palette {
    Palette::LIGHT
}

fn parses(markup: &str) -> bool {
    usvg::Tree::from_str(markup, &usvg::Options::default()).is_ok()
}

fn view_box(markup: &str) -> Vec<f64> {
    let rest = markup.split("viewBox=\"").nth(1).unwrap();
    rest.split('"')
        .next()
        .unwrap()
        .split_whitespace()
        .map(|v| v.parse().unwrap())
        .collect()
}

/// A golden file: compares, or writes it when `KANSO_BLESS` is set.
fn golden(name: &str, actual: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/canvas/embed/golden")
        .join(name);
    if std::env::var_os("KANSO_BLESS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("no golden {name}; run with KANSO_BLESS=1"));
    assert_eq!(expected, actual, "{name} changed; KANSO_BLESS=1 to accept");
}

#[test]
fn a_named_group_is_the_area() {
    let svg = draw(&sample(), &spec(Some("g1")), &palette()).unwrap();
    assert!(parses(&svg.markup));
    assert_eq!(view_box(&svg.markup), vec![-20.0, -20.0, 300.0, 160.0]);
    assert!(svg.markup.contains("Launch plan"));
    assert!(!svg.markup.contains("Far away"));
    assert!(svg.notices.is_empty());
}

#[test]
fn without_a_group_the_content_is_the_area() {
    let svg = draw(&sample(), &spec(None), &palette()).unwrap();
    let b = sample().content_bounds().unwrap();
    let v = view_box(&svg.markup);
    assert!(v[0] < b.x && v[1] < b.y && v[0] + v[2] > b.right() && v[1] + v[3] > b.bottom());
    assert!(svg.markup.contains("Launch plan") && svg.markup.contains("Far away"));
    assert!(svg.notices.is_empty());
}

#[test]
fn a_deleted_group_falls_back_to_the_content_with_a_notice() {
    let mut canvas = sample();
    assert!(canvas.remove_node("g1", 9));
    let svg = draw(&canvas, &spec(Some("g1")), &palette()).unwrap();
    assert!(svg.markup.contains("Far away"));
    assert_eq!(svg.notices, vec![Notice::GroupMissing]);
    let unknown = draw(&sample(), &spec(Some("nope")), &palette()).unwrap();
    assert_eq!(unknown.notices, vec![Notice::GroupMissing]);
}

#[test]
fn an_empty_canvas_is_a_small_card_with_its_title() {
    let svg = draw(&Canvas::default(), &spec(None), &palette()).unwrap();
    assert!(parses(&svg.markup));
    assert_eq!((svg.width, svg.height), (320.0, 200.0));
    assert!(svg.markup.contains("Roadmap") && svg.markup.contains("Empty canvas"));
    let wide = draw(
        &Canvas::default(),
        &Spec {
            width: Some(640),
            ..spec(None)
        },
        &palette(),
    )
    .unwrap();
    assert_eq!((wide.width, wide.height), (640.0, 400.0));
}

#[test]
fn the_width_sets_the_size_and_keeps_the_shape() {
    let plain = draw(&sample(), &spec(Some("g1")), &palette()).unwrap();
    let wide = draw(
        &sample(),
        &Spec {
            width: Some(600),
            ..spec(Some("g1"))
        },
        &palette(),
    )
    .unwrap();
    assert_eq!(wide.width, 600.0);
    assert!((wide.height / wide.width - plain.height / plain.width).abs() < 1e-3);
}

#[test]
fn a_huge_whole_canvas_still_draws_with_a_notice() {
    let mut canvas = sample();
    canvas.put_node(card("far", 20_000.0, 0.0, 100.0, 50.0, "Out there"), 5);
    let whole = draw(&canvas, &spec(None), &palette()).unwrap();
    assert!(parses(&whole.markup));
    assert_eq!(whole.notices, vec![Notice::WholeCanvas]);
    // Naming a group is the way out, and it is not warned about.
    let part = draw(&canvas, &spec(Some("g1")), &palette()).unwrap();
    assert!(part.notices.is_empty());
    // Just under the limit is not huge.
    let mut small = sample();
    small.put_node(card("edge", 7_000.0, 0.0, 100.0, 50.0, "Near"), 5);
    assert!(
        draw(&small, &spec(None), &palette())
            .unwrap()
            .notices
            .is_empty()
    );
}

#[test]
fn the_preview_block_is_stable() {
    let svg = draw(&sample(), &spec(Some("g1")), &palette()).unwrap();
    golden("preview_block.svg", &svg.markup);
    let dark = draw(&sample(), &spec(Some("g1")), &Palette::DARK).unwrap();
    golden("preview_block_dark.svg", &dark.markup);
}

#[test]
fn the_huge_embed_is_stable() {
    let mut canvas = sample();
    canvas.put_node(card("far", 20_000.0, 0.0, 100.0, 50.0, "Out there"), 5);
    let svg = draw(&canvas, &spec(None), &palette()).unwrap();
    golden("huge_embed.svg", &svg.markup);
    golden("huge_embed_notice.txt", Notice::WholeCanvas.text());
}

#[test]
fn the_empty_card_is_stable() {
    let svg = draw(&Canvas::default(), &spec(None), &palette()).unwrap();
    golden("empty_card.svg", &svg.markup);
}

fn notes_of<'a>(
    canvases: &'a HashMap<String, (String, String)>,
    asked: &'a Cell<usize>,
) -> impl Fn(&str) -> Option<EmbedNote> + 'a {
    move |id| {
        asked.set(asked.get() + 1);
        canvases.get(id).map(|(title, body)| EmbedNote::Open {
            title: title.clone(),
            body: body.as_str().into(),
        })
    }
}

#[test]
fn show_finds_the_note_and_says_why_not() {
    let canvases = HashMap::from([("note:a".to_owned(), ("A".to_owned(), to_body(&sample())))]);
    let asked = Cell::new(0);
    let notes = notes_of(&canvases, &asked);
    let drawn = show(&Embed::of("note:a").at_group("g1"), &notes, &palette());
    assert!(matches!(drawn, Shown::Drawing(_)));
    assert_eq!(
        show(&Embed::of("note:zz"), &notes, &palette()),
        Shown::Missing
    );
    let broken = HashMap::from([(
        "note:b".to_owned(),
        ("B".to_owned(), "```canvas\n{nope\n```".to_owned()),
    )]);
    let notes = notes_of(&broken, &asked);
    assert!(matches!(
        show(&Embed::of("note:b"), &notes, &palette()),
        Shown::Invalid(_)
    ));
    let other = |_: &str| Some(EmbedNote::NotCanvas);
    assert_eq!(
        show(&Embed::of("note:c"), &other, &palette()),
        Shown::NotCanvas
    );
}

#[test]
fn a_locked_canvas_is_a_placeholder_and_is_never_cached() {
    let secret = "Secret merger plan";
    let mut canvas = Canvas::default();
    canvas.put_node(card("n1", 0.0, 0.0, 240.0, 100.0, secret), 1);
    let body = to_body(&canvas);
    let locked = |_: &str| Some(EmbedNote::Locked);
    let shown = show(&Embed::of("note:l"), &locked, &palette());
    assert_eq!(shown, Shown::Locked);
    assert_eq!(placeholder_text(&shown), Some("Locked canvas"));
    // Nothing of the canvas was drawn, so nothing of it is kept.
    let job = Job::canvas(body.as_str(), spec(None), palette());
    assert!(crate::cached(&job).is_none());
}

fn text() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{4,20}"
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Invariant: a locked canvas never renders content: whatever it holds,
    /// the embed is the placeholder, and nothing it holds reaches the
    /// output or the render cache.
    #[test]
    fn prop_a_locked_canvas_never_renders_content(
        words in prop::collection::vec(text(), 1..6),
        group_name in prop::option::of("[a-z]{1,4}"),
        width in prop::option::of(16u32..2000),
    ) {
        let mut canvas = Canvas::default();
        for (i, word) in words.iter().enumerate() {
            canvas.put_node(card(&format!("n{i}"), i as f64 * 300.0, 0.0, 240.0, 100.0, word), 1);
        }
        let body = to_body(&canvas);
        let locked = |_: &str| Some(EmbedNote::Locked);
        let embed = Embed { note: "note:l".into(), group: group_name.clone(), width };
        let shown = show(&embed, &locked, &palette());
        prop_assert_eq!(shown, Shown::Locked);
        let job = Job::canvas(body.as_str(), Spec { title: "T".into(), group: group_name, width }, palette());
        prop_assert!(crate::cached(&job).is_none());
    }

    /// Invariant: embeds cannot recurse. However canvases point at each
    /// other and at themselves, showing an embed asks for its note exactly
    /// once, ends, and draws note cards as their titles only.
    #[test]
    fn prop_cycles_never_recurse(
        count in 1usize..6,
        links in prop::collection::vec((0usize..6, 0usize..6), 0..14),
    ) {
        let ids: Vec<String> = (0..count).map(|i| format!("note:c{i}")).collect();
        let mut canvases: HashMap<String, (String, String)> = HashMap::new();
        for (i, id) in ids.iter().enumerate() {
            let mut canvas = Canvas::default();
            canvas.put_node(card("t", 0.0, 0.0, 200.0, 80.0, &format!("Body of canvas {i}")), 1);
            for (n, (from, to)) in links.iter().enumerate() {
                if from % count == i {
                    canvas.put_node(
                        Node {
                            id: format!("l{n}"),
                            kind: "file".into(),
                            file: Some(ids[to % count].clone()),
                            title: Some(format!("Title of {}", to % count)),
                            x: 300.0 + n as f64 * 220.0,
                            y: 0.0,
                            width: 200.0,
                            height: 80.0,
                            ..Node::default()
                        },
                        1,
                    );
                }
            }
            canvases.insert(id.clone(), (format!("Canvas {i}"), to_body(&canvas)));
        }
        for (i, id) in ids.iter().enumerate() {
            let asked = Cell::new(0);
            let notes = notes_of(&canvases, &asked);
            let shown = show(&Embed::of(id.clone()), &notes, &palette());
            prop_assert_eq!(asked.get(), 1);
            let Shown::Drawing(svg) = shown else {
                return Err(TestCaseError::fail("a canvas didn't draw"));
            };
            let own = format!("Body of canvas {i}");
            prop_assert!(svg.markup.contains(&own));
            for j in (0..count).filter(|j| *j != i) {
                let other = format!("Body of canvas {j}");
                prop_assert!(!svg.markup.contains(&other));
            }
        }
    }

    /// Invariant: whatever area is asked for, an embed draws a valid SVG of
    /// the size asked, or fails with a reason; it never panics.
    #[test]
    fn prop_any_area_draws_or_explains(
        x in -500.0f64..500.0, y in -500.0f64..500.0,
        w in 0.0f64..900.0, h in 0.0f64..900.0,
        width in prop::option::of(16u32..2000),
        named in prop::bool::ANY,
    ) {
        let mut canvas = sample();
        canvas.put_node(group("gx", x, y, w, h), 4);
        let spec = Spec { title: "T".into(), group: named.then(|| "gx".into()), width };
        if let Ok(svg) = draw(&canvas, &spec, &palette()) {
            prop_assert!(parses(&svg.markup));
            prop_assert!(svg.width > 0.0 && svg.height > 0.0);
            if let Some(width) = width {
                prop_assert!((svg.width - width as f32).abs() < 0.01);
            }
        } else {
            prop_assert!(named && (w == 0.0 || h == 0.0));
        }
    }
}

fn page_with_embed(notes: Notes<'_>, fence: &str) -> String {
    let options = crate::html::PageOptions {
        notes: Some(notes),
        ..crate::html::PageOptions::default()
    };
    let none = |_: &str| None;
    crate::html::body_with(fence, &palette(), &none, None, &options)
}

#[test]
fn a_page_inlines_the_canvas_and_a_locked_one_is_a_placeholder() {
    let body: Arc<str> = to_body(&sample()).into();
    let open = |_: &str| {
        Some(EmbedNote::Open {
            title: "T".into(),
            body: body.clone(),
        })
    };
    let html = page_with_embed(&open, "```canvas\nnote:abc\ngroup:g1\n```\n");
    assert!(html.contains("kanso-canvas") && html.contains("Launch plan"));
    assert!(!html.contains("Far away") && !html.contains("<script"));
    let locked = |_: &str| Some(EmbedNote::Locked);
    let html = page_with_embed(&locked, "```canvas\nnote:abc\n```\n");
    golden("locked_placeholder.html", &html);
    assert!(html.contains("Locked canvas") && !html.contains("Launch plan"));
    // Without a resolver the fence stays code.
    let plain = crate::html::body("```canvas\nnote:abc\n```\n", &palette(), &|_: &str| None);
    assert!(!plain.contains("kanso-canvas"));
}

#[test]
fn a_note_that_embeds_itself_ends() {
    let body: Arc<str> = "```canvas\nnote:me\n```".into();
    let me = |_: &str| {
        Some(EmbedNote::Open {
            title: "Me".into(),
            body: body.clone(),
        })
    };
    let html = page_with_embed(&me, "```canvas\nnote:me\n```\n");
    assert!(
        html.contains("Couldn&#39;t draw this canvas")
            || html.contains("Couldn't draw this canvas")
    );
}
