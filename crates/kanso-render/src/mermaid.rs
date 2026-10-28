//! Mermaid diagrams in the preview theme's colors, drawn by Pele, or by the
//! pure-Rust `mermaid-svg` (Settings › Editor › Preview).

use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};

use mermaid_svg::{ParseError, RenderError, Theme};

use crate::{Failure, Outcome, Palette, Rgb, Svg, svg};

/// The longest diagram source drawn.
const MAX_SOURCE: usize = 64 * 1024;
/// Room around a diagram, in points.
const PAD: f32 = 8.;

/// How big arrowheads are drawn, against `mermaid-svg`'s own: its heads are
/// heavy beside the thin lines and crowd the nodes they point at.
const MARKER_SCALE: f32 = 0.6;
/// SVG's size for a marker that doesn't give one.
const DEFAULT_MARKER_SIZE: f32 = 3.;

/// `svg` with every `<marker>` drawn at `scale` of its size. Markers keep
/// their view box and reference point, so a smaller head still ends where
/// the line does. Pure.
fn shrink_markers(svg: &str, scale: f32) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(at) = rest.find("<marker") {
        let after = &rest[at + "<marker".len()..];
        // `<markerX` is some other tag; only `<marker` followed by space or `>`.
        if !after.starts_with(|c: char| c.is_whitespace() || c == '>' || c == '/') {
            out.push_str(&rest[..at + "<marker".len()]);
            rest = after;
            continue;
        }
        let Some(end) = after.find('>') else { break };
        out.push_str(&rest[..at]);
        let mut tag = after[..end].to_owned();
        for name in ["markerWidth", "markerHeight"] {
            tag = scale_attribute(&tag, name, scale);
        }
        out.push_str("<marker");
        out.push_str(&tag);
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// `attributes` (a tag's text after its name) with `name="n"` scaled, or
/// added at the scaled default size when absent.
fn scale_attribute(attributes: &str, name: &str, scale: f32) -> String {
    let key = format!("{name}=\"");
    let found = attributes.match_indices(&key).find(|(at, _)| {
        attributes[..*at]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
    });
    match found {
        Some((at, _)) => {
            let start = at + key.len();
            let Some(len) = attributes[start..].find('"') else {
                return attributes.to_owned();
            };
            let Ok(value) = attributes[start..start + len].trim().parse::<f32>() else {
                return attributes.to_owned();
            };
            format!(
                "{}{}{}",
                &attributes[..start],
                svg::number(value * scale),
                &attributes[start + len..]
            )
        }
        None => {
            let (body, closing) = match attributes.strip_suffix('/') {
                Some(body) => (body, "/"),
                None => (attributes, ""),
            };
            format!(
                "{body} {name}=\"{}\"{closing}",
                svg::number(DEFAULT_MARKER_SIZE * scale)
            )
        }
    }
}

/// The diagram keywords `mermaid-svg` draws.
const KEYWORDS: &[&str] = &[
    "pie",
    "sequenceDiagram",
    "flowchart",
    "graph",
    "flowchart-elk",
    "stateDiagram",
    "stateDiagram-v2",
    "classDiagram",
    "classDiagram-v2",
    "erDiagram",
    "gantt",
    "journey",
    "timeline",
    "sankey",
    "sankey-beta",
    "quadrantChart",
    "xychart",
    "xychart-beta",
    "radar",
    "radar-beta",
    "packet",
    "packet-beta",
    "mindmap",
    "gitGraph",
    "requirementDiagram",
    "C4Context",
    "C4Container",
    "C4Component",
    "C4Dynamic",
    "C4Deployment",
    "block",
    "block-beta",
    "architecture",
    "architecture-beta",
    "kanban",
    "treemap",
    "treemap-beta",
    "zenuml",
];

/// The diagram's keyword: the first word after any front matter, `%%`
/// comments and `%%{init}%%` directives.
fn keyword(source: &str) -> &str {
    let mut lines = source.lines().map(str::trim);
    let mut in_front_matter = false;
    for line in lines.by_ref() {
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }
        if line == "---" {
            in_front_matter = !in_front_matter;
            continue;
        }
        if in_front_matter {
            continue;
        }
        return line
            .split(|c: char| c.is_whitespace() || c == ';')
            .next()
            .unwrap_or("")
            .trim_end_matches(':');
    }
    ""
}

/// Whether Kanso draws diagrams like `source`.
pub fn is_supported(source: &str) -> bool {
    KEYWORDS.contains(&keyword(source))
}

/// The theme: the page's colors, nodes tinted with its accent.
fn theme(p: &Palette) -> Theme {
    let base = if p.dark {
        Theme::dark()
    } else {
        Theme::default_theme()
    };
    let hex = |c: Rgb| Cow::Owned(c.hex());
    let node_fill = p
        .accent
        .over(p.background, if p.dark { 0.16 } else { 0.10 });
    let node_stroke = p.accent.over(p.background, 0.6);
    let line = p.muted.over(p.text, 0.5);
    let note = Rgb(0xE9C46A).over(p.background, if p.dark { 0.22 } else { 0.28 });
    Theme {
        fg: hex(p.text),
        fg_muted: hex(p.muted),
        bg: hex(p.background),
        actor_fill: hex(node_fill),
        actor_stroke: hex(node_stroke),
        lifeline: hex(p.border.over(p.muted, 0.5)),
        arrow_stroke: hex(line),
        note_fill: hex(note),
        note_stroke: hex(Rgb(0xC9A227).over(p.background, 0.6)),
        activation_fill: hex(node_fill),
        activation_stroke: hex(node_stroke),
        frame_label_fill: hex(node_fill),
        flow_node_fill: hex(node_fill),
        flow_node_stroke: hex(node_stroke),
        flow_edge_stroke: hex(line),
        flow_label_bg: hex(p.surface),
        flow_cluster_fill: hex(p.surface),
        flow_cluster_stroke: hex(p.border),
        pie_stroke: Some(hex(p.background)),
        font_family: Cow::Borrowed(
            "'Helvetica Neue', 'Segoe UI', Inter, 'Noto Sans', 'DejaVu Sans', Arial, sans-serif",
        ),
        font_size: 14.,
        responsive: false,
        ..base
    }
}

/// What draws a diagram.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Engine {
    /// `mermaid-svg`, in Rust.
    MermaidSvg,
    /// Pele, Obsidian's renderer, in QuickJS: the default.
    #[default]
    Pele,
}

/// Whether new jobs draw with Pele.
static PELE: AtomicBool = AtomicBool::new(true);

/// Chooses the engine new [`crate::Job`]s draw Mermaid with.
pub fn set_engine(engine: Engine) {
    PELE.store(engine == Engine::Pele, Ordering::Relaxed);
}

/// The engine new jobs draw with.
pub fn engine() -> Engine {
    if PELE.load(Ordering::Relaxed) {
        Engine::Pele
    } else {
        Engine::MermaidSvg
    }
}

/// Draws a Mermaid diagram with `engine`.
pub fn render_with(engine: Engine, source: &str, palette: &Palette) -> Outcome {
    if source.len() > MAX_SOURCE {
        return Err(Failure::Invalid("The diagram is too long to draw.".into()));
    }
    if engine == Engine::Pele {
        let markup = crate::pele::theme(&crate::pele::render(source)?, palette);
        return finish(&markup);
    }
    let theme = theme(palette);
    // A third-party layout engine: a bug in it shows the source, not a crash.
    let drawn = std::panic::catch_unwind(|| mermaid_svg::render_with(source, &theme))
        .map_err(|_| Failure::Invalid("The diagram couldn't be laid out.".into()))?;
    let markup = drawn.map_err(|err| match err {
        RenderError::Parse(ParseError::UnknownDiagramType(kind)) => Failure::Unsupported(kind),
        RenderError::Parse(ParseError::Empty) => Failure::Invalid("The diagram is empty.".into()),
        RenderError::Parse(err) => Failure::Invalid(err.to_string()),
        #[allow(unreachable_patterns)]
        err => Failure::Invalid(err.to_string()),
    })?;
    finish(&shrink_markers(&markup, MARKER_SCALE))
}

/// A drawing's SVG, flattened and cropped.
fn finish(markup: &str) -> Outcome {
    let flat = svg::flatten(markup, PAD)
        .ok_or_else(|| Failure::Invalid("The diagram drew nothing.".into()))?;
    Ok(Svg {
        markup: flat.markup,
        width: flat.width,
        height: flat.height,
        baseline: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Drawn by `mermaid-svg`, whatever new jobs would use.
    fn render(source: &str, palette: &Palette) -> Outcome {
        render_with(Engine::MermaidSvg, source, palette)
    }

    const SAMPLES: &[&str] = &[
        "flowchart TD\n  A[Start] --> B{Working?}\n  B -->|Yes| C[Ship]\n  B -->|No| D[Debug] --> B\n",
        "graph LR\n  subgraph Client\n    a[Editor] --> b[Preview]\n  end\n  b --> c[(Store)]\n",
        "sequenceDiagram\n  Alice->>Bob: Hello\n  Bob-->>Alice: Hi\n  Note over Alice,Bob: A note\n",
        "classDiagram\n  Animal <|-- Dog\n  class Animal { +String name }\n",
        "stateDiagram-v2\n  [*] --> Still\n  Still --> Moving\n  Moving --> [*]\n",
        "erDiagram\n  CUSTOMER ||--o{ ORDER : places\n",
        "pie title Pets\n  \"Dogs\" : 386\n  \"Cats\" : 85\n",
        "gantt\n  dateFormat YYYY-MM-DD\n  section A\n  Task :a1, 2024-01-01, 3d\n",
        "mindmap\n  root((Kanso))\n    Notes\n    Sync\n",
        "---\ntitle: T\n---\n%% a comment\njourney\n  title Day\n  section Work\n    Write: 5: Me\n",
    ];

    #[test]
    fn each_engine_is_cached_on_its_own() {
        let job = crate::Job::new(
            crate::Kind::Mermaid,
            "flowchart LR\n  a --> b\n",
            Palette::LIGHT,
        );
        let pele = crate::Job {
            mermaid: Engine::Pele,
            ..job.clone()
        };
        let mermaid_svg = crate::Job {
            mermaid: Engine::MermaidSvg,
            ..job
        };
        assert_ne!(pele, mermaid_svg);
        let (a, b) = (
            crate::render_blocking(&pele),
            crate::render_blocking(&mermaid_svg),
        );
        assert_ne!(
            a.as_ref().as_ref().unwrap().markup,
            b.as_ref().as_ref().unwrap().markup
        );
    }

    #[test]
    fn common_diagrams_draw_in_the_theme() {
        for source in SAMPLES {
            for palette in [Palette::LIGHT, Palette::DARK] {
                let svg = render(source, &palette).unwrap_or_else(|e| panic!("{source}: {e:?}"));
                assert!(svg.width > 2. * PAD && svg.height > 2. * PAD, "{source}");
                assert!(!svg.markup.contains("<text"), "text is drawn as outlines");
                assert!(!svg.markup.contains("foreignObject"));
                let tree = usvg::Tree::from_str(&svg.markup, &usvg::Options::default()).unwrap();
                assert!((tree.size().width() - svg.width).abs() < 0.01);
            }
            assert!(is_supported(source), "{source}");
        }
        let dark = render(SAMPLES[0], &Palette::DARK).unwrap();
        assert!(
            dark.markup
                .contains(&Palette::DARK.text.over(Palette::DARK.muted, 0.5).hex())
                || dark
                    .markup
                    .contains(&Palette::DARK.muted.over(Palette::DARK.text, 0.5).hex())
        );
    }

    #[test]
    fn unknown_kinds_and_mistakes_are_told_apart() {
        assert_eq!(
            render("sparkleDiagram\n  a --> b\n", &Palette::LIGHT),
            Err(Failure::Unsupported("sparkleDiagram".into()))
        );
        assert!(!is_supported("sparkleDiagram\n"));
        assert!(matches!(
            render("", &Palette::LIGHT),
            Err(Failure::Invalid(_))
        ));
        assert_eq!(keyword("%%{init: {}}%%\n\n  graph TD;A-->B"), "graph");
        assert_eq!(keyword("gitGraph:\n commit"), "gitGraph");
    }

    #[test]
    fn arrowheads_are_drawn_smaller() {
        let svg = "<svg><defs><marker id=\"a\" viewBox=\"0 0 10 10\" refX=\"10\" \
                   markerWidth=\"10\" markerHeight=\"8\" orient=\"auto\"><path/></marker>\
                   <marker id=\"b\"/></defs><markerish/></svg>";
        let shrunk = shrink_markers(svg, 0.6);
        assert!(shrunk.contains("markerWidth=\"6\" markerHeight=\"4.8\""));
        assert!(shrunk.contains("viewBox=\"0 0 10 10\" refX=\"10\""));
        assert!(shrunk.contains("<marker id=\"b\" markerWidth=\"1.8\" markerHeight=\"1.8\"/>"));
        assert!(shrunk.contains("<markerish/>"));
        // A real flowchart's heads shrink too, and still parse.
        let source = "flowchart LR\n  A --> B\n";
        let raw = mermaid_svg::render_with(source, &theme(&Palette::LIGHT)).unwrap();
        let shrunk = shrink_markers(&raw, MARKER_SCALE);
        assert_ne!(raw, shrunk);
        assert!(usvg::Tree::from_str(&shrunk, &usvg::Options::default()).is_ok());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Invariants: shrinking leaves everything but marker sizes alone
        /// (so markup without markers is unchanged), and a size scales by
        /// exactly the factor.
        #[test]
        fn prop_markers_scale_exactly(size in 1u32..40, plain in "[a-z >=\"/]{0,30}") {
            prop_assert_eq!(shrink_markers(&plain, 0.6), plain.clone());
            let svg = format!("{plain}<marker markerWidth=\"{size}\" markerHeight=\"{size}\">");
            let expected = svg::number(size as f32 * 0.5);
            let want = format!("<marker markerWidth=\"{expected}\" markerHeight=\"{expected}\">");
            prop_assert!(shrink_markers(&svg, 0.5).ends_with(&want));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Invariant: whatever the text, drawing answers without panicking,
        /// and a drawing is a sized, parseable SVG.
        #[test]
        fn prop_drawing_answers(body in prop::collection::vec("[A-Za-z0-9 \\[\\]{}()|>-]{0,16}", 0..8), head in prop::sample::select(vec!["flowchart TD", "graph LR", "sequenceDiagram", "pie", "classDiagram", "stateDiagram-v2", "nonsense"])) {
            let source = format!("{head}\n{}", body.join("\n"));
            match render(&source, &Palette::LIGHT) {
                Ok(svg) => {
                    prop_assert!(svg.width > 0. && svg.height > 0.);
                    prop_assert!(usvg::Tree::from_str(&svg.markup, &usvg::Options::default()).is_ok());
                }
                Err(Failure::Unsupported(kind)) => prop_assert_eq!(kind, "nonsense"),
                Err(Failure::Invalid(_) | Failure::NoPlugin(_)) => {}
            }
        }
    }
}
