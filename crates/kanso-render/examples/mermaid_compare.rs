//! Spike: draws the same Mermaid diagrams with `mermaid-svg` and with Pele,
//! in the light and dark palettes, into one HTML page to compare by eye.
//! The SVGs are the flattened ones the app shows, text as outlines.
//!
//! `cargo run -p kanso-render --example mermaid_compare --release -- out.html`

use std::fmt::Write as _;
use std::time::Instant;

use kanso_render::{MermaidEngine, Palette, render_mermaid_with};

const SAMPLES: &[(&str, &str)] = &[
    (
        "flowchart",
        "flowchart TD\n  A[Start] --> B{Working?}\n  B -->|Yes| C[Ship]\n  B -->|No| D[Debug] --> B\n",
    ),
    (
        "flowchart, subgraph",
        "graph LR\n  subgraph Client\n    a[Editor] --> b[Preview]\n  end\n  b --> c[(Store)]\n  b -.-> d([Sync])\n",
    ),
    (
        "flowchart, Kanso's",
        "flowchart TD\n  A[Christmas] -->|Get money| B(Go shopping)\n  B --> C{Let me think}\n  C -->|One| D[Laptop]\n  C -->|Two| E[iPhone]\n  C -->|Three| F[Car]\n",
    ),
    (
        "sequence",
        "sequenceDiagram\n  actor Alice\n  participant Bob\n  Alice->>Bob: Hello\n  Bob-->>Alice: Hi\n  Note over Alice,Bob: A note\n  loop Every minute\n    Alice->>Bob: Ping\n  end\n  alt ok\n    Bob->>Alice: Pong\n  else not\n    Bob-xAlice: Error\n  end\n",
    ),
    (
        "class",
        "classDiagram\n  Animal <|-- Dog\n  Animal <|-- Cat\n  class Animal {\n    +String name\n    +eat() void\n  }\n  class Dog {\n    +bark()\n  }\n",
    ),
    (
        "state",
        "stateDiagram-v2\n  [*] --> Still\n  Still --> Moving\n  Moving --> Still\n  Moving --> Crash\n  Crash --> [*]\n",
    ),
    (
        "er",
        "erDiagram\n  CUSTOMER ||--o{ ORDER : places\n  ORDER ||--|{ LINE-ITEM : contains\n  CUSTOMER {\n    string name\n    string email\n  }\n",
    ),
    (
        "pie",
        "pie title Pets\n  \"Dogs\" : 386\n  \"Cats\" : 85\n  \"Rats\" : 15\n",
    ),
    (
        "gantt",
        "gantt\n  title Release\n  dateFormat YYYY-MM-DD\n  section Build\n  Design :a1, 2026-01-01, 5d\n  Code :crit, a2, after a1, 7d\n  section Ship\n  Review :after a2, 3d\n",
    ),
    (
        "mindmap",
        "mindmap\n  root((Kanso))\n    Notes\n      Markdown\n      Locked\n    Sync\n    Themes\n",
    ),
    (
        "journey",
        "journey\n  title Day\n  section Work\n    Write: 5: Me\n    Review: 3: Me, Bob\n",
    ),
    (
        "gitGraph",
        "gitGraph\n  commit\n  branch dev\n  commit\n  commit\n  checkout main\n  merge dev\n  commit\n",
    ),
    (
        "timeline",
        "timeline\n  title Kanso\n  2025 : Started\n  2026 : GPUI : Sync\n",
    ),
    (
        "quadrant",
        "quadrantChart\n  title Effort\n  x-axis Low --> High\n  y-axis Small --> Big\n  A: [0.3, 0.6]\n  B: [0.7, 0.2]\n",
    ),
    (
        "xychart",
        "xychart-beta\n  title Notes\n  x-axis [Jan, Feb, Mar, Apr]\n  bar [3, 7, 5, 9]\n  line [3, 7, 5, 9]\n",
    ),
    (
        "sankey",
        "sankey-beta\nNotes,Synced,10\nNotes,Local,5\nSynced,Phone,4\n",
    ),
    (
        "kanban",
        "kanban\n  Todo\n    a[Write spike]\n  Doing\n    b[Compare]\n  Done\n    c[Merge 34]\n",
    ),
    (
        "block",
        "block-beta\n  columns 3\n  a[Editor] b[Preview] c[Store]\n",
    ),
    (
        "architecture",
        "architecture-beta\n  group app(cloud)[App]\n  service db(database)[DB] in app\n  service api(server)[API] in app\n  db:L -- R:api\n",
    ),
    (
        "C4",
        "C4Context\n  Person(a, \"User\")\n  System(b, \"Kanso\")\n  Rel(a, b, \"Uses\")\n",
    ),
    (
        "treemap",
        "treemap-beta\n\"Notes\"\n  \"Work\": 10\n  \"Home\": 5\n",
    ),
    (
        "math in labels",
        "flowchart LR\n  A[\"$$x^2 + \\frac{a}{b}$$\"] --> B[\"Sum $$\\sum_{i=1}^{n} i$$ done\"]\n  B --> C[\"$$\\int_0^1 f(x)\\,dx$$\"]\n",
    ),
    ("unsupported", "zenuml\n  Alice->Bob: hi\n"),
];

fn cell(engine: MermaidEngine, source: &str, palette: &Palette, page: &mut String) -> f64 {
    let started = Instant::now();
    let outcome = render_mermaid_with(engine, source, palette);
    let ms = started.elapsed().as_secs_f64() * 1000.;
    let bg = palette.background.hex();
    let fg = palette.text.hex();
    write!(page, "<td style=\"background:{bg};color:{fg}\">").unwrap();
    match outcome {
        Ok(svg) => write!(
            page,
            "{}<div class=t>{ms:.1} ms · {:.0}×{:.0}</div>",
            svg.markup, svg.width, svg.height
        ),
        Err(failure) => write!(
            page,
            "<div class=f>{} {}</div><div class=t>{ms:.1} ms</div>",
            failure.badge(),
            kanso_render_escape(failure.detail())
        ),
    }
    .unwrap();
    page.push_str("</td>");
    ms
}

fn kanso_render_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;")
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "mermaid-compare.html".into());
    // Start both engines first, so the first diagram's time is a diagram's.
    let warm = Instant::now();
    render_mermaid_with(MermaidEngine::Pele, "flowchart LR\n a-->b", &Palette::LIGHT).ok();
    let pele_start = warm.elapsed().as_secs_f64() * 1000.;

    let mut page = String::from(
        "<!doctype html><meta charset=utf-8><title>Mermaid: mermaid-svg vs Pele</title>\
         <style>body{font:13px -apple-system,sans-serif;margin:16px}table{border-collapse:collapse}\
         td,th{border:1px solid #ccc;padding:8px;vertical-align:top}th{text-align:left}\
         td svg{max-width:420px;height:auto}.t{font-size:11px;opacity:.6;margin-top:4px}\
         .f{color:#c0504d}</style><h1>Mermaid: mermaid-svg vs Pele 1.9.0</h1>",
    );
    let mut body = String::new();
    let mut totals = [0f64; 2];
    for (name, source) in SAMPLES {
        for palette in [Palette::LIGHT, Palette::DARK] {
            write!(
                body,
                "<tr><th>{name}<br><small>{}</small></th>",
                if palette.dark { "dark" } else { "light" }
            )
            .unwrap();
            totals[0] += cell(MermaidEngine::MermaidSvg, source, &palette, &mut body);
            totals[1] += cell(MermaidEngine::Pele, source, &palette, &mut body);
            body.push_str("</tr>");
        }
    }
    writeln!(
        page,
        "<p>Pele's engine started in {pele_start:.0} ms. All {} drawings: mermaid-svg {:.0} ms, \
         Pele {:.0} ms (each including flattening).</p>\
         <table><tr><th></th><th>mermaid-svg</th><th>Pele</th></tr>{body}</table>",
        SAMPLES.len() * 2,
        totals[0],
        totals[1]
    )
    .unwrap();
    std::fs::write(&out, page).expect("writing the page");
    println!(
        "{out}: Pele started in {pele_start:.0} ms; mermaid-svg {:.0} ms, Pele {:.0} ms",
        totals[0], totals[1]
    );
}
