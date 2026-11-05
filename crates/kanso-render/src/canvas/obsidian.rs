//! A canvas as an Obsidian `.canvas` file and the files its cards point at:
//! cards, text and arrows stay editable there, each layer of ink becomes a
//! `file` card of a PNG (or is left out, "cards only"), and each attachment
//! card brings its file along.

use std::collections::HashSet;

use kanso_core::canvas::{Canvas, ExportFiles, INK};

use super::{Scene, render};
use crate::{Palette, svg};

/// Pixels to the point in an ink PNG.
const PNG_SCALE: f32 = 2.0;

/// The longest side of an ink PNG, in pixels.
pub const PNG_MAX_SIDE: u32 = 4096;

/// The start of an ink PNG's file name; its number follows.
const INK_FILE_PREFIX: &str = "Ink ";

/// What an export does with ink.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InkExport {
    /// Each layer of ink becomes a PNG file card at the layer's place.
    Png,
    /// Ink is left out.
    CardsOnly,
}

/// An attachment a canvas card points at, with its bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attachment {
    /// The attachment's document ID, as the card holds it.
    pub id: String,
    /// The name it had in Kanso.
    pub name: String,
    pub data: Vec<u8>,
}

/// The files an export writes into its folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exported {
    /// The `.canvas` file's text.
    pub canvas: String,
    /// Every other file, by the name the `.canvas` file gives it: ink PNGs
    /// and attachments.
    pub files: Vec<(String, Vec<u8>)>,
    /// Layers of ink that couldn't be drawn and so were left out.
    pub ink_left_out: usize,
}

/// `canvas` as a `.canvas` file with the files it needs. `attachments` are
/// the attachments the canvas might point at; only the ones its live cards
/// use are written. Pure.
pub fn export(canvas: &Canvas, ink: InkExport, attachments: &[Attachment]) -> Exported {
    let mut files = ExportFiles::default();
    let mut written: Vec<(String, Vec<u8>)> = Vec::new();
    let mut taken: HashSet<String> = HashSet::new();
    for id in canvas.attachment_ids() {
        let Some(attachment) = attachments.iter().find(|a| a.id == id) else {
            continue;
        };
        let name = unique_name(&file_name(&attachment.name), &mut taken);
        files.attachments.insert(id, name.clone());
        written.push((name, attachment.data.clone()));
    }
    let mut ink_left_out = 0;
    if ink == InkExport::Png {
        let mut n = 0;
        for node in canvas.live_nodes().filter(|n| n.kind == INK) {
            if node.strokes.iter().all(|s| s.deleted) {
                continue;
            }
            let Some(png) = png_of(node) else {
                ink_left_out += 1;
                continue;
            };
            n += 1;
            let name = unique_name(&format!("{INK_FILE_PREFIX}{n}.png"), &mut taken);
            files.ink.insert(node.id.clone(), name.clone());
            written.push((name, png));
        }
    }
    Exported {
        canvas: canvas.to_obsidian_json(&files),
        files: written,
        ink_left_out,
    }
}

/// One layer of ink as a PNG on a clear background, in the pens' own colours.
fn png_of(node: &kanso_core::canvas::Node) -> Option<Vec<u8>> {
    let layer = Canvas {
        nodes: vec![node.clone()],
        ..Canvas::default()
    };
    let scene = Scene {
        area: Some(node.bounds()),
        transparent: true,
        ..Scene::default()
    };
    let drawing = render(&layer, &scene, &Palette::LIGHT).ok()?;
    svg::rasterize(&drawing.markup, PNG_SCALE, PNG_MAX_SIDE)
}

/// `name` as a file name: no folders, nothing a file system refuses.
fn file_name(name: &str) -> String {
    let last = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let cleaned: String = last
        .chars()
        .map(|c| match c {
            ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let cleaned = cleaned.trim().trim_start_matches('.').trim();
    if cleaned.is_empty() {
        "attachment".into()
    } else {
        cleaned.into()
    }
}

/// `name`, or `name (2)`, `name (3)`… before its extension, whichever
/// `taken` (compared without regard to case) doesn't hold yet; taken after.
fn unique_name(name: &str, taken: &mut HashSet<String>) -> String {
    let (stem, ext) = match name.rfind('.') {
        Some(at) if at > 0 => (&name[..at], &name[at..]),
        _ => (name, ""),
    };
    let mut candidate = name.to_owned();
    let mut n = 2;
    while !taken.insert(candidate.to_lowercase()) {
        candidate = format!("{stem} ({n}){ext}");
        n += 1;
    }
    candidate
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use kanso_core::canvas::{Canvas, Node, Point, Stroke};
    use proptest::prelude::*;
    use serde_json::Value;

    use super::*;

    fn png_size(png: &[u8]) -> (u32, u32) {
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "not a PNG");
        let at = |i: usize| u32::from_be_bytes(png[i..i + 4].try_into().unwrap());
        (at(16), at(20))
    }

    fn stroke(x: f64, span: f64) -> Stroke {
        Stroke {
            tool: "pen".into(),
            color: "#e03131".into(),
            width: 3.0,
            points: (0..20)
                .map(|i| Point::new(x + i as f64 * span / 20.0, (i as f64).sin() * 20.0, 0.7))
                .collect(),
            ..Stroke::default()
        }
    }

    fn card(id: &str, kind: &str, file: Option<&str>) -> Node {
        Node {
            id: id.into(),
            kind: kind.into(),
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 80.0,
            text: (kind == "text").then(|| "hello".into()),
            file: file.map(str::to_owned),
            title: file.map(|_| "Plan".into()),
            ..Node::default()
        }
    }

    fn board() -> Canvas {
        let mut c = Canvas::default();
        c.put_node(card("t", "text", None), 1);
        c.put_node(card("n", "file", Some("note:abc")), 2);
        c.put_node(card("p", "file", Some("file:pic")), 3);
        c.put_edge(
            kanso_core::canvas::Edge {
                id: "e".into(),
                from_node: "t".into(),
                to_node: "n".into(),
                ..Default::default()
            },
            4,
        );
        c.draw(stroke(0.0, 300.0), 5).unwrap();
        c
    }

    fn pic() -> Attachment {
        Attachment {
            id: "file:pic".into(),
            name: "folder/pic.png".into(),
            data: b"\x89PNG-bytes".to_vec(),
        }
    }

    fn cards_in(json: &str) -> Vec<Value> {
        let value: Value = serde_json::from_str(json).unwrap();
        value["nodes"].as_array().unwrap().clone()
    }

    #[test]
    fn ink_becomes_a_png_file_card() {
        let out = export(&board(), InkExport::Png, &[pic()]);
        assert_eq!(out.ink_left_out, 0);
        let names: Vec<&str> = out.files.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["pic.png", "Ink 1.png"]);
        let (_, png) = &out.files[1];
        let (w, h) = png_size(png);
        assert!(w > 100 && h > 10 && w.max(h) <= PNG_MAX_SIDE);
        let nodes = cards_in(&out.canvas);
        assert_eq!(nodes.len(), 4);
        let ink = nodes.iter().find(|n| n["file"] == "Ink 1.png").unwrap();
        assert_eq!(ink["type"], "file");
        assert!(ink["width"].is_i64() && ink["height"].is_i64());
        let attached = nodes.iter().find(|n| n["file"] == "pic.png").unwrap();
        assert_eq!(attached["kansoId"], "file:pic");
        let note = nodes.iter().find(|n| n["id"] == "n").unwrap();
        assert_eq!(note["file"], "Plan.md");
        // The file the card points at comes back in as a canvas.
        let back = Canvas::from_obsidian(&out.canvas).unwrap();
        assert_eq!(back.canvas.nodes.len(), 4);
        assert_eq!(back.canvas.edges.len(), 1);
    }

    #[test]
    fn cards_only_leaves_ink_out() {
        let out = export(&board(), InkExport::CardsOnly, &[pic()]);
        assert_eq!(out.files.len(), 1, "just the attachment");
        assert_eq!(cards_in(&out.canvas).len(), 3);
        assert!(!out.canvas.contains("Ink 1.png"));
        assert_eq!(out.ink_left_out, 0);
    }

    #[test]
    fn a_huge_layer_of_ink_is_cut_to_the_png_cap() {
        let mut c = Canvas::default();
        c.draw(stroke(0.0, 20_000.0), 1).unwrap();
        let out = export(&c, InkExport::Png, &[]);
        let (_, png) = &out.files[0];
        let (w, h) = png_size(png);
        assert!(w.max(h) <= PNG_MAX_SIDE, "{w}x{h}");
        assert!(w > h);
    }

    #[test]
    fn attachment_names_are_safe_and_distinct() {
        let mut c = Canvas::default();
        c.put_node(card("a", "file", Some("file:1")), 1);
        c.put_node(card("b", "file", Some("file:2")), 2);
        c.put_node(card("c", "file", Some("file:3")), 3);
        let attach = |id: &str, name: &str| Attachment {
            id: id.into(),
            name: name.into(),
            data: vec![1],
        };
        let out = export(
            &c,
            InkExport::CardsOnly,
            &[
                attach("file:1", "../a:b.png"),
                attach("file:2", "A-B.PNG"),
                attach("file:3", "a-b.png"),
                attach("file:unused", "unused.png"),
            ],
        );
        let names: Vec<&str> = out.files.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["a-b.png", "A-B (2).PNG", "a-b (3).png"]);
    }

    fn arb_canvas() -> impl Strategy<Value = Canvas> {
        let pts = prop::collection::vec((-3000f64..3000., -3000f64..3000., 0f64..=1.), 1..40);
        (
            prop::collection::vec(prop::sample::select(vec!["text", "file", "group"]), 0..6),
            prop::collection::vec(pts, 0..4),
        )
            .prop_map(|(kinds, strokes)| {
                let mut c = Canvas::default();
                for (i, kind) in kinds.into_iter().enumerate() {
                    let file = match (kind, i % 3) {
                        ("file", 0) => Some("note:n".to_owned()),
                        ("file", _) => Some(format!("file:{}", i % 3)),
                        _ => None,
                    };
                    c.put_node(card(&format!("c{i}"), kind, file.as_deref()), 1);
                }
                for pts in strokes {
                    let s = Stroke {
                        tool: "pen".into(),
                        color: "#336699".into(),
                        width: 2.0,
                        points: pts
                            .into_iter()
                            .map(|(x, y, p)| Point::new(x, y, p))
                            .collect(),
                        ..Stroke::default()
                    };
                    let _ = c.draw(s, 2);
                }
                c
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(16))]

        /// Invariants: an export is always a file that imports as a canvas;
        /// every file a card names was written, once, under a distinct
        /// name; every ink PNG is a PNG within the cap; "cards only" writes
        /// no ink; ink that was drawn is not silently lost.
        #[test]
        fn prop_an_export_is_a_canvas_with_all_its_files(
            canvas in arb_canvas(),
            names in prop::collection::vec("[a-zA-Z0-9 ./:]{0,10}", 3),
        ) {
            let attachments: Vec<Attachment> = names
                .iter()
                .enumerate()
                .map(|(i, n)| Attachment { id: format!("file:{i}"), name: n.clone(), data: vec![i as u8] })
                .collect();
            let live_ink = canvas
                .live_nodes()
                .filter(|n| n.kind == INK && n.strokes.iter().any(|s| !s.deleted))
                .count();
            for mode in [InkExport::Png, InkExport::CardsOnly] {
                let out = export(&canvas, mode, &attachments);
                prop_assert!(Canvas::from_obsidian(&out.canvas).is_ok());
                let distinct: HashSet<String> =
                    out.files.iter().map(|(n, _)| n.to_lowercase()).collect();
                prop_assert_eq!(distinct.len(), out.files.len());
                let written: HashSet<&str> = out.files.iter().map(|(n, _)| n.as_str()).collect();
                let mut pngs = 0;
                for node in cards_in(&out.canvas) {
                    if node["kansoId"].as_str().is_some_and(|id| id.starts_with("file:")) {
                        let file = node["file"].as_str().unwrap();
                        prop_assert!(written.contains(file), "{file} not written");
                    }
                    if node["file"].as_str().is_some_and(|f| f.starts_with(INK_FILE_PREFIX)) {
                        pngs += 1;
                    }
                }
                for (name, data) in &out.files {
                    if name.starts_with(INK_FILE_PREFIX) {
                        let (w, h) = png_size(data);
                        prop_assert!(w.max(h) <= PNG_MAX_SIDE && w > 0 && h > 0);
                    }
                }
                match mode {
                    InkExport::CardsOnly => prop_assert_eq!(pngs, 0),
                    InkExport::Png => prop_assert_eq!(pngs + out.ink_left_out, live_ink),
                }
            }
        }
    }
}
