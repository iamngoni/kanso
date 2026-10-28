//! The note graph: which notes link to which, a force-directed layout of
//! it, and the view transform the graph view draws it through. All pure;
//! the apps draw it.
//!
//! The graph comes from the store's `note_links` index: ID links by ID, and wiki-links by title, resolved as everywhere else to
//! the first-created live note with that title. Links are undirected for
//! drawing: A→B and B→A are one edge. A note linking to itself keeps its
//! self-loop (it is real), but is never its own neighbour.
//!
//! The layout is Fruchterman–Reingold on a grid: neighbours on the grid
//! repel, edges attract, a little gravity keeps islands near the middle,
//! and a temperature that cools to nothing over a fixed number of steps
//! means it settles instead of jittering. It is deterministic for a seed,
//! so the same library always draws the same picture.

use std::collections::{HashMap, HashSet, VecDeque};

use kanso_core::TagColor;
use kanso_core::links::normalize_title;
use kanso_store::{GraphLink, GraphNote, Store};

use crate::theme::muted_tag_color;

/// Which graph a view shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// The open note and its neighbours.
    Local,
    /// Every note.
    Library,
}

/// What the library's graph is narrowed to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum GraphFilter {
    #[default]
    All,
    /// A notebook and the notebooks inside it.
    Book(String),
    Tag(String),
}

/// The seed every layout of the graph is made with, so a library always
/// draws the same picture.
pub const SEED: u64 = 0x6B61_6E73_6F67_7261;

/// The length an edge settles towards, in layout units.
pub const IDEAL_EDGE: f32 = 42.;

/// A note in the graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub title: String,
    pub book_id: String,
    pub tags: Vec<String>,
}

/// Notes and the links between them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Graph {
    pub nodes: Vec<Node>,
    /// Undirected, as `(a, b)` with `a <= b`, each once, sorted. `a == b`
    /// is a note linking to itself.
    pub edges: Vec<(u32, u32)>,
    /// Each node's neighbours, sorted, never itself.
    pub neighbours: Vec<Vec<u32>>,
}

impl Graph {
    /// The graph of `notes` (first created first) and the `links` their
    /// bodies make. Links to notes that aren't there are left out.
    pub fn build(notes: Vec<GraphNote>, links: &[GraphLink]) -> Graph {
        let mut by_id: HashMap<&str, u32> = HashMap::with_capacity(notes.len());
        let mut by_title: HashMap<String, u32> = HashMap::with_capacity(notes.len());
        for (ix, note) in notes.iter().enumerate() {
            by_id.insert(note.id.as_str(), ix as u32);
            let title = normalize_title(&note.title);
            if !title.is_empty() {
                // The first created keeps the title.
                by_title.entry(title).or_insert(ix as u32);
            }
        }
        let mut edges = HashSet::new();
        for link in links {
            let Some(&from) = by_id.get(link.from.as_str()) else {
                continue;
            };
            let to = match (&link.target_id, &link.target_title) {
                (Some(id), _) => by_id.get(id.as_str()).copied(),
                (None, Some(title)) => by_title.get(title.as_str()).copied(),
                (None, None) => None,
            };
            if let Some(to) = to {
                edges.insert((from.min(to), from.max(to)));
            }
        }
        let nodes = notes
            .into_iter()
            .map(|n| Node {
                id: n.id,
                title: n.title,
                book_id: n.book_id,
                tags: n.tags,
            })
            .collect();
        Graph::from_edges(nodes, edges.into_iter().collect())
    }

    /// The graph of `nodes` with `edges` (undirected, deduplicated here).
    pub fn from_edges(nodes: Vec<Node>, mut edges: Vec<(u32, u32)>) -> Graph {
        for edge in &mut edges {
            *edge = (edge.0.min(edge.1), edge.0.max(edge.1));
        }
        edges.retain(|&(a, b)| (b as usize) < nodes.len() && a <= b);
        edges.sort_unstable();
        edges.dedup();
        let mut neighbours = vec![Vec::new(); nodes.len()];
        for &(a, b) in &edges {
            if a != b {
                neighbours[a as usize].push(b);
                neighbours[b as usize].push(a);
            }
        }
        for list in &mut neighbours {
            list.sort_unstable();
        }
        Graph {
            nodes,
            edges,
            neighbours,
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The node of note `id`.
    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.id == id)
    }

    /// How many other notes node `ix` links with.
    pub fn degree(&self, ix: usize) -> usize {
        self.neighbours[ix].len()
    }

    /// Node `center` and the nodes up to `hops` links from it, in order.
    pub fn neighbourhood(&self, center: usize, hops: u8) -> Vec<usize> {
        let mut seen = vec![false; self.len()];
        let mut found = vec![center];
        seen[center] = true;
        let mut queue = VecDeque::from([(center, 0u8)]);
        while let Some((ix, depth)) = queue.pop_front() {
            if depth == hops {
                continue;
            }
            for &next in &self.neighbours[ix] {
                let next = next as usize;
                if !seen[next] {
                    seen[next] = true;
                    found.push(next);
                    queue.push_back((next, depth + 1));
                }
            }
        }
        found.sort_unstable();
        found
    }

    /// The graph of the nodes `keep` (indices into this graph, in order)
    /// and the edges among them.
    pub fn subgraph(&self, keep: &[usize]) -> Graph {
        let mut new_index = vec![u32::MAX; self.len()];
        for (new, &old) in keep.iter().enumerate() {
            new_index[old] = new as u32;
        }
        let nodes = keep.iter().map(|&ix| self.nodes[ix].clone()).collect();
        let edges = self
            .edges
            .iter()
            .filter_map(|&(a, b)| {
                let (a, b) = (new_index[a as usize], new_index[b as usize]);
                (a != u32::MAX && b != u32::MAX).then_some((a, b))
            })
            .collect();
        Graph::from_edges(nodes, edges)
    }

    /// The nodes for which `keep` holds, and the edges among them.
    pub fn filtered(&self, keep: impl Fn(&Node) -> bool) -> Graph {
        let indices: Vec<usize> = (0..self.len())
            .filter(|&ix| keep(&self.nodes[ix]))
            .collect();
        self.subgraph(&indices)
    }

    /// How many links the graph draws, self-loops left out.
    pub fn link_count(&self) -> usize {
        self.edges.iter().filter(|(a, b)| a != b).count()
    }
}

/// A small fast deterministic random source (SplitMix64).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// In `-1..1`.
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 23) as f32 - 1.
    }
}

/// How far a layout of `n` nodes may reach from the middle: no node ever
/// goes further, in either axis.
pub fn layout_bound(n: usize) -> f32 {
    IDEAL_EDGE * (4. + 3. * (n as f32).sqrt())
}

/// How many steps a layout of `n` nodes takes: fewer for big graphs, whose
/// grid keeps each step cheap but whose steps still add up.
pub fn layout_steps(n: usize) -> u32 {
    match n {
        0..=1 => 0,
        2..=200 => 300,
        201..=1000 => 220,
        _ => 160,
    }
}

/// Lays `graph` out, deterministically for `seed`: positions in layout
/// units, centred near the origin, every one finite and within
/// [`layout_bound`].
pub fn layout(graph: &Graph, seed: u64) -> Vec<[f32; 2]> {
    let n = graph.len();
    let k = IDEAL_EDGE;
    let bound = layout_bound(n);
    let mut rng = Rng(seed ^ (n as u64).wrapping_mul(0x2545_F491_4F6C_DD1D));
    // A sunflower spiral, jittered: spread out evenly, so the first steps
    // don't fling crowded nodes apart.
    let golden = std::f32::consts::PI * (3. - 5f32.sqrt());
    let mut pos: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let r = k * 0.9 * (i as f32 + 0.5).sqrt();
            let a = i as f32 * golden + rng.unit() * 0.3;
            [r * a.cos() + rng.unit() * 2., r * a.sin() + rng.unit() * 2.]
        })
        .collect();
    let steps = layout_steps(n);
    if steps == 0 {
        return pos.into_iter().map(|_| [0., 0.]).collect();
    }
    let cell = 2. * k;
    let reach2 = cell * cell;
    let mut shift = vec![[0f32; 2]; n];
    let mut grid: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
    let start_temp = k * 2.5;
    for step in 0..steps {
        let temp = start_temp * (1. - step as f32 / steps as f32);
        shift.fill([0., 0.]);
        grid.clear();
        let cell_of = |p: [f32; 2]| ((p[0] / cell).floor() as i32, (p[1] / cell).floor() as i32);
        for (ix, p) in pos.iter().enumerate() {
            grid.entry(cell_of(*p)).or_default().push(ix as u32);
        }
        // Repulsion from the nodes in the neighbouring cells.
        for i in 0..n {
            let (cx, cy) = cell_of(pos[i]);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let Some(others) = grid.get(&(cx + dx, cy + dy)) else {
                        continue;
                    };
                    for &j in others {
                        let j = j as usize;
                        if j == i {
                            continue;
                        }
                        let (mut ddx, mut ddy) = (pos[i][0] - pos[j][0], pos[i][1] - pos[j][1]);
                        let mut d2 = ddx * ddx + ddy * ddy;
                        if d2 > reach2 {
                            continue;
                        }
                        if d2 < 1e-4 {
                            // On top of each other: apart in a direction
                            // that only depends on the pair.
                            let a = ((i * 7919 + j * 104_729) % 360) as f32;
                            ddx = a.to_radians().cos() * 0.01;
                            ddy = a.to_radians().sin() * 0.01;
                            d2 = 1e-4;
                        }
                        let force = k * k / d2;
                        shift[i][0] += ddx * force;
                        shift[i][1] += ddy * force;
                    }
                }
            }
        }
        // Attraction along the edges.
        for &(a, b) in &graph.edges {
            let (a, b) = (a as usize, b as usize);
            if a == b {
                continue;
            }
            let (dx, dy) = (pos[a][0] - pos[b][0], pos[a][1] - pos[b][1]);
            let d = (dx * dx + dy * dy).sqrt();
            let force = d / k;
            shift[a][0] -= dx * force;
            shift[a][1] -= dy * force;
            shift[b][0] += dx * force;
            shift[b][1] += dy * force;
        }
        // Gravity, and the move, no further than the temperature allows.
        for (p, s) in pos.iter_mut().zip(&mut shift) {
            s[0] -= p[0] * 0.04;
            s[1] -= p[1] * 0.04;
            let len = (s[0] * s[0] + s[1] * s[1]).sqrt();
            if len.is_finite() && len > 0. {
                let step = len.min(temp);
                p[0] = (p[0] + s[0] / len * step).clamp(-bound, bound);
                p[1] = (p[1] + s[1] / len * step).clamp(-bound, bound);
            }
        }
    }
    // Centred on the middle of the nodes.
    let (mut mx, mut my) = (0f32, 0f32);
    for p in &pos {
        mx += p[0];
        my += p[1];
    }
    let (mx, my) = (mx / n as f32, my / n as f32);
    for p in &mut pos {
        p[0] = (p[0] - mx).clamp(-bound, bound);
        p[1] = (p[1] - my).clamp(-bound, bound);
    }
    pos
}

/// Turns `positions` about their middle so their longest spread runs up
/// and down: the inspector's pane is tall and narrow, so a graph standing
/// that way is drawn bigger. A rotation only: every distance is kept.
pub fn stand_tall(positions: &mut [[f32; 2]]) {
    let n = positions.len();
    if n < 2 {
        return;
    }
    let (mut mx, mut my) = (0f64, 0f64);
    for p in positions.iter() {
        mx += f64::from(p[0]);
        my += f64::from(p[1]);
    }
    let (mx, my) = (mx / n as f64, my / n as f64);
    let (mut xx, mut yy, mut xy) = (0f64, 0f64, 0f64);
    for p in positions.iter() {
        let (dx, dy) = (f64::from(p[0]) - mx, f64::from(p[1]) - my);
        xx += dx * dx;
        yy += dy * dy;
        xy += dx * dy;
    }
    // The principal axis's angle, then the turn that makes it vertical.
    let axis = 0.5 * (2. * xy).atan2(xx - yy);
    let turn = std::f64::consts::FRAC_PI_2 - axis;
    if !turn.is_finite() {
        return;
    }
    let (sin, cos) = turn.sin_cos();
    for p in positions.iter_mut() {
        let (dx, dy) = (f64::from(p[0]) - mx, f64::from(p[1]) - my);
        p[0] = (mx + dx * cos - dy * sin) as f32;
        p[1] = (my + dx * sin + dy * cos) as f32;
    }
}

/// The rectangle around `positions`, as `(min, max)`; `None` for none.
pub fn extent(positions: &[[f32; 2]]) -> Option<([f32; 2], [f32; 2])> {
    let first = *positions.first()?;
    Some(positions.iter().fold((first, first), |(lo, hi), p| {
        (
            [lo[0].min(p[0]), lo[1].min(p[1])],
            [hi[0].max(p[0]), hi[1].max(p[1])],
        )
    }))
}

/// How the layout maps onto the screen: the layout point in the middle of
/// the view, and how many points one layout unit takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub center: [f32; 2],
    pub zoom: f32,
}

/// How far the view zooms in and out.
pub const ZOOM_RANGE: (f32, f32) = (0.05, 6.);

impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            center: [0., 0.],
            zoom: 1.,
        }
    }
}

impl Viewport {
    /// The view that shows all of `positions` in a view `size` points big,
    /// with `margin` points around them, no closer in than `max_zoom`.
    pub fn fit(positions: &[[f32; 2]], size: [f32; 2], margin: f32, max_zoom: f32) -> Viewport {
        let Some((lo, hi)) = extent(positions) else {
            return Viewport::default();
        };
        let (w, h) = ((hi[0] - lo[0]).max(1.), (hi[1] - lo[1]).max(1.));
        let room = [
            (size[0] - 2. * margin).max(1.),
            (size[1] - 2. * margin).max(1.),
        ];
        let zoom = (room[0] / w)
            .min(room[1] / h)
            .clamp(ZOOM_RANGE.0, max_zoom.min(ZOOM_RANGE.1));
        Viewport {
            center: [(lo[0] + hi[0]) / 2., (lo[1] + hi[1]) / 2.],
            zoom,
        }
    }

    /// Where layout point `p` is in a view `size` big, from its top left.
    pub fn to_screen(self, p: [f32; 2], size: [f32; 2]) -> [f32; 2] {
        [
            (p[0] - self.center[0]) * self.zoom + size[0] / 2.,
            (p[1] - self.center[1]) * self.zoom + size[1] / 2.,
        ]
    }

    /// The layout point at `s` in a view `size` big.
    pub fn to_layout(self, s: [f32; 2], size: [f32; 2]) -> [f32; 2] {
        [
            (s[0] - size[0] / 2.) / self.zoom + self.center[0],
            (s[1] - size[1] / 2.) / self.zoom + self.center[1],
        ]
    }

    /// Zooms by `factor` (within [`ZOOM_RANGE`]), keeping the layout point
    /// under `at` where it is.
    pub fn zoom_at(&self, factor: f32, at: [f32; 2], size: [f32; 2]) -> Viewport {
        let anchor = self.to_layout(at, size);
        let zoom = (self.zoom * factor).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
        let mut next = Viewport {
            center: self.center,
            zoom,
        };
        let moved = next.to_layout(at, size);
        next.center[0] += anchor[0] - moved[0];
        next.center[1] += anchor[1] - moved[1];
        next
    }

    /// Moves the view by `delta` points, as dragging the graph does.
    pub fn panned(&self, delta: [f32; 2]) -> Viewport {
        Viewport {
            center: [
                self.center[0] - delta[0] / self.zoom,
                self.center[1] - delta[1] / self.zoom,
            ],
            zoom: self.zoom,
        }
    }

    /// The layout rectangle a view `size` big shows, grown by `margin`
    /// points on every side.
    pub fn visible(&self, size: [f32; 2], margin: f32) -> ([f32; 2], [f32; 2]) {
        (
            self.to_layout([-margin, -margin], size),
            self.to_layout([size[0] + margin, size[1] + margin], size),
        )
    }
}

/// Whether a segment from `a` to `b` may cross the rectangle `(lo, hi)`:
/// its own bounding box meets it.
pub fn segment_visible(a: [f32; 2], b: [f32; 2], (lo, hi): ([f32; 2], [f32; 2])) -> bool {
    a[0].max(b[0]) >= lo[0]
        && a[0].min(b[0]) <= hi[0]
        && a[1].max(b[1]) >= lo[1]
        && a[1].min(b[1]) <= hi[1]
}

/// A node's radius in layout units: bigger the more it links.
pub fn node_radius(degree: usize) -> f32 {
    3.5 + 1.6 * (degree as f32).sqrt().min(8.)
}

/// The node under `at` (screen points), if any: the nearest whose circle,
/// grown by `slop` points, holds it.
pub fn hit(
    positions: &[[f32; 2]],
    radii: &[f32],
    view: &Viewport,
    size: [f32; 2],
    at: [f32; 2],
    slop: f32,
) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (ix, p) in positions.iter().enumerate() {
        let s = view.to_screen(*p, size);
        let d = ((s[0] - at[0]).powi(2) + (s[1] - at[1]).powi(2)).sqrt();
        let reach = screen_radius(radii[ix], view.zoom) + slop;
        if d <= reach && best.is_none_or(|(_, b)| d < b) {
            best = Some((ix, d));
        }
    }
    best.map(|(ix, _)| ix)
}

/// A node's radius on screen at `zoom`: it shrinks as the view zooms out,
/// but never below a dot nor above a button.
pub fn screen_radius(radius: f32, zoom: f32) -> f32 {
    (radius * zoom.sqrt()).clamp(1.6, 16.)
}

/// Whether notebook `id` is `book` or inside it, by `books`' `(id, parent)`
/// pairs: up the parents, at most as deep as there are notebooks.
pub fn in_notebook(id: &str, book: &str, books: &[(String, Option<String>)]) -> bool {
    let mut current = Some(id);
    for _ in 0..=books.len() {
        let Some(at) = current else { return false };
        if at == book {
            return true;
        }
        current = books
            .iter()
            .find(|(b, _)| b == at)
            .and_then(|(_, parent)| parent.as_deref());
    }
    false
}

/// A notebook's color: one of the tag colors, picked by its ID, so it stays
/// the same between runs and devices; calmed for the theme like tags are.
pub fn book_color(book_id: &str, dark: bool) -> u32 {
    // Not blue: that is the accent, which marks the open note.
    const HUES: [TagColor; 10] = [
        TagColor::Brown,
        TagColor::Green,
        TagColor::Orange,
        TagColor::Violet,
        TagColor::Teal,
        TagColor::Red,
        TagColor::Yellow,
        TagColor::Pink,
        TagColor::Olive,
        TagColor::Purple,
    ];
    // FNV-1a: stable, unlike the standard library's hasher.
    let hash = book_id.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    muted_tag_color(HUES[(hash % HUES.len() as u64) as usize].rgb(), dark)
}

/// A graph laid out, ready to draw: the nodes shown, where each goes in
/// layout units and how big each is.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Arranged {
    pub graph: Graph,
    pub pos: Vec<[f32; 2]>,
    pub radii: Vec<f32>,
}

/// The graph of every note in `store`, with the locked ones (sealed, or in
/// a locked notebook) only if `show_locked`: the lock password has opened
/// them this session. A link to a note that is left out is left out too.
pub fn read(store: &Store, show_locked: bool) -> kanso_store::Result<Graph> {
    let (notes, links) = store.graph_rows(show_locked)?;
    Ok(Graph::build(notes, &links))
}

/// What `full` shows for a view, laid out with [`SEED`]: for the local
/// scope the neighbourhood `hops` links around `center` (nothing without a
/// center, or when the center isn't in the graph), standing tall; for the
/// library's the notes `filter` keeps, with `books` as `(id, parent)` pairs.
pub fn arrange(
    full: Graph,
    scope: Scope,
    center: Option<&str>,
    hops: u8,
    filter: &GraphFilter,
    books: &[(String, Option<String>)],
) -> Arranged {
    let graph = match (scope, center) {
        (Scope::Local, Some(center)) => match full.index_of(center) {
            Some(ix) => full.subgraph(&full.neighbourhood(ix, hops)),
            None => Graph::default(),
        },
        (Scope::Local, None) => Graph::default(),
        (Scope::Library, _) => match filter {
            GraphFilter::All => full,
            GraphFilter::Book(book) => full.filtered(|n| in_notebook(&n.book_id, book, books)),
            GraphFilter::Tag(tag) => full.filtered(|n| n.tags.contains(tag)),
        },
    };
    let mut pos = layout(&graph, SEED);
    if scope == Scope::Local {
        stand_tall(&mut pos);
    }
    let radii = (0..graph.len())
        .map(|ix| node_radius(graph.degree(ix)))
        .collect();
    Arranged { graph, pos, radii }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn note(id: &str, title: &str) -> GraphNote {
        GraphNote {
            id: id.into(),
            title: title.into(),
            book_id: "book:a".into(),
            tags: Vec::new(),
        }
    }

    fn by_id(from: &str, to: &str) -> GraphLink {
        GraphLink {
            from: from.into(),
            target_id: Some(to.into()),
            target_title: None,
        }
    }

    fn by_title(from: &str, to: &str) -> GraphLink {
        GraphLink {
            from: from.into(),
            target_id: None,
            target_title: Some(normalize_title(to)),
        }
    }

    fn nodes(n: usize) -> Vec<Node> {
        (0..n)
            .map(|i| Node {
                id: format!("note:{i}"),
                title: format!("Note {i}"),
                book_id: String::new(),
                tags: Vec::new(),
            })
            .collect()
    }

    #[test]
    fn links_resolve_by_id_and_by_the_first_title() {
        let notes = vec![
            note("note:a", "Plan"),
            note("note:b", "Plan"),
            note("note:c", "Other"),
        ];
        let links = [
            by_title("note:c", "  plan "),
            by_id("note:b", "note:c"),
            by_id("note:c", "note:b"),
            by_id("note:a", "note:gone"),
            by_title("note:a", "Nowhere"),
            by_id("note:a", "note:a"),
        ];
        let graph = Graph::build(notes, &links);
        // C→Plan resolves to A, the first; B↔C is one edge; A→A is real.
        assert_eq!(graph.edges, vec![(0, 0), (0, 2), (1, 2)]);
        assert_eq!(graph.neighbours[0], vec![2]);
        assert_eq!(graph.neighbours[2], vec![0, 1]);
        assert_eq!(graph.link_count(), 2);
        assert_eq!(graph.neighbourhood(1, 1), vec![1, 2]);
        assert_eq!(graph.neighbourhood(1, 2), vec![0, 1, 2]);
        let sub = graph.subgraph(&[1, 2]);
        assert_eq!(sub.edges, vec![(0, 1)]);
    }

    #[test]
    fn a_big_library_lays_out_bounded() {
        // 2,000 notes in chains with cross links, as a large library.
        let n = 2_000;
        let edges: Vec<(u32, u32)> = (1..n as u32)
            .map(|i| (i - 1, i))
            .chain((0..n as u32).step_by(7).map(|i| (i, (i * 13) % n as u32)))
            .collect();
        let graph = Graph::from_edges(nodes(n), edges);
        let pos = layout(&graph, 7);
        let bound = layout_bound(n);
        assert!(
            pos.iter()
                .all(|p| p.iter().all(|x| x.is_finite() && x.abs() <= bound))
        );
    }

    fn arb_graph(max: usize) -> impl Strategy<Value = Graph> {
        (0..max).prop_flat_map(|n| {
            let edges = if n == 0 {
                Just(Vec::new()).boxed()
            } else {
                prop::collection::vec((0..n as u32, 0..n as u32), 0..n * 2).boxed()
            };
            edges.prop_map(move |edges| Graph::from_edges(nodes(n), edges))
        })
    }

    proptest! {
        #![proptest_config(config(64))]

        /// Every position is finite and within the bound, and the same
        /// seed lays out the same picture.
        #[test]
        fn prop_layout_is_finite_bounded_and_deterministic(graph in arb_graph(40), seed: u64) {
            let pos = layout(&graph, seed);
            prop_assert_eq!(pos.len(), graph.len());
            let bound = layout_bound(graph.len());
            for p in &pos {
                prop_assert!(p[0].is_finite() && p[1].is_finite());
                prop_assert!(p[0].abs() <= bound && p[1].abs() <= bound);
            }
            prop_assert_eq!(pos, layout(&graph, seed));
        }

        /// Linked notes end up nearer each other than the layout's
        /// average pair does.
        #[test]
        fn prop_linked_notes_settle_close(n in 6usize..30, seed: u64) {
            // A ring: every note links to the next.
            let edges = (0..n as u32).map(|i| (i, (i + 1) % n as u32)).collect();
            let graph = Graph::from_edges(nodes(n), edges);
            let pos = layout(&graph, seed);
            let d = |a: usize, b: usize| ((pos[a][0] - pos[b][0]).powi(2) + (pos[a][1] - pos[b][1]).powi(2)).sqrt();
            let linked: f32 = graph.edges.iter().map(|&(a, b)| d(a as usize, b as usize)).sum::<f32>() / graph.edges.len() as f32;
            let mut all = 0f32;
            let mut pairs = 0f32;
            for a in 0..n { for b in a + 1..n { all += d(a, b); pairs += 1.; } }
            prop_assert!(linked < all / pairs);
        }

        /// Building from links: edges are undirected and unique, a link and
        /// its reverse are one edge, neighbours are symmetric, and the
        /// only self-loops are notes that link to themselves.
        #[test]
        fn prop_graph_building_is_symmetric(
            n in 1usize..12,
            links in prop::collection::vec((0usize..14, 0usize..14, any::<bool>()), 0..40),
        ) {
            let notes: Vec<GraphNote> = (0..n).map(|i| note(&format!("note:{i}"), &format!("T{}", i % 4))).collect();
            let raw: Vec<GraphLink> = links.iter().map(|&(a, b, title)| {
                if title { by_title(&format!("note:{a}"), &format!("t{}", b % 5)) } else { by_id(&format!("note:{a}"), &format!("note:{b}")) }
            }).collect();
            let reversed: Vec<GraphLink> = links.iter().map(|&(a, b, title)| {
                if title { by_title(&format!("note:{a}"), &format!("t{}", b % 5)) } else { by_id(&format!("note:{b}"), &format!("note:{a}")) }
            }).collect();
            let graph = Graph::build(notes.clone(), &raw);
            // The model: an ID names its note; "tK" names the first note
            // titled "TK", which is note K.
            let resolve = |b: usize, title: bool| if title { (b % 5 < 4 && b % 5 < n).then_some(b % 5) } else { (b < n).then_some(b) };
            let mut expected: Vec<(u32, u32)> = links.iter().filter_map(|&(a, b, title)| {
                let to = resolve(b, title)?;
                (a < n).then_some((a.min(to) as u32, a.max(to) as u32))
            }).collect();
            expected.sort_unstable();
            expected.dedup();
            prop_assert_eq!(&graph.edges, &expected);
            for &(a, b) in &graph.edges {
                if a != b {
                    prop_assert!(graph.neighbours[a as usize].contains(&b));
                    prop_assert!(graph.neighbours[b as usize].contains(&a));
                }
            }
            for (ix, list) in graph.neighbours.iter().enumerate() {
                prop_assert!(!list.contains(&(ix as u32)));
            }
            // ID links point either way to the same edges.
            let other = Graph::build(notes, &reversed);
            prop_assert_eq!(graph.edges, other.edges);
        }

        /// Standing a layout up keeps every distance, and leaves it no
        /// wider than it is tall.
        #[test]
        fn prop_stand_tall_rotates_to_tall(points in prop::collection::vec((-500f32..500., -500f32..500.), 2..30)) {
            let before: Vec<[f32; 2]> = points.iter().map(|&(x, y)| [x, y]).collect();
            let mut after = before.clone();
            stand_tall(&mut after);
            let dist = |a: [f32; 2], b: [f32; 2]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
            for i in 0..before.len() {
                prop_assert!(after[i][0].is_finite() && after[i][1].is_finite());
                for j in 0..before.len() {
                    prop_assert!((dist(before[i], before[j]) - dist(after[i], after[j])).abs() < 0.05);
                }
            }
            let spread = |axis: usize| {
                let m = after.iter().map(|p| p[axis]).sum::<f32>() / after.len() as f32;
                after.iter().map(|p| (p[axis] - m).powi(2)).sum::<f32>()
            };
            prop_assert!(spread(0) <= spread(1) * 1.001 + 1e-2);
        }

        /// The view maps back and forth, and zooming keeps the point under
        /// the pointer still.
        #[test]
        fn prop_viewport_round_trips(
            cx in -500f32..500., cy in -500f32..500., zoom in 0.05f32..6.,
            px in -800f32..800., py in -800f32..800.,
            ax in 0f32..900., ay in 0f32..700., factor in 0.2f32..5.,
        ) {
            let view = Viewport { center: [cx, cy], zoom };
            let size = [900., 700.];
            let back = view.to_layout(view.to_screen([px, py], size), size);
            prop_assert!((back[0] - px).abs() < 0.05 && (back[1] - py).abs() < 0.05);
            let anchor = view.to_layout([ax, ay], size);
            let zoomed = view.zoom_at(factor, [ax, ay], size);
            prop_assert!((ZOOM_RANGE.0..=ZOOM_RANGE.1).contains(&zoomed.zoom));
            let still = zoomed.to_layout([ax, ay], size);
            prop_assert!((still[0] - anchor[0]).abs() < 0.05 && (still[1] - anchor[1]).abs() < 0.05);
        }

        /// Fitting shows every position inside the view.
        #[test]
        fn prop_fit_shows_everything(points in prop::collection::vec((-2000f32..2000., -2000f32..2000.), 1..40)) {
            let pos: Vec<[f32; 2]> = points.into_iter().map(|(x, y)| [x, y]).collect();
            let size = [600., 400.];
            let view = Viewport::fit(&pos, size, 20., 2.);
            if view.zoom > ZOOM_RANGE.0 {
                for p in &pos {
                    let s = view.to_screen(*p, size);
                    prop_assert!(s[0] >= 19. && s[0] <= 581. && s[1] >= 19. && s[1] <= 381.);
                }
            }
        }
    }

    #[test]
    fn notebooks_are_inside_themselves_and_their_children() {
        let books = vec![
            ("a".to_owned(), None),
            ("b".to_owned(), Some("a".to_owned())),
            ("c".to_owned(), Some("b".to_owned())),
            ("d".to_owned(), None),
        ];
        assert!(in_notebook("c", "a", &books));
        assert!(in_notebook("a", "a", &books));
        assert!(!in_notebook("a", "c", &books));
        assert!(!in_notebook("d", "a", &books));
        assert!(!in_notebook("gone", "a", &books));
    }

    #[test]
    fn a_notebook_inside_itself_does_not_loop() {
        let books = vec![
            ("a".to_owned(), Some("b".to_owned())),
            ("b".to_owned(), Some("a".to_owned())),
        ];
        assert!(!in_notebook("a", "z", &books));
    }

    #[test]
    fn the_local_graph_is_the_neighbourhood_and_the_library_graph_is_filtered() {
        let notes = vec![
            note("note:a", "A"),
            note("note:b", "B"),
            note("note:c", "C"),
        ];
        let full = Graph::build(notes, &[by_id("note:a", "note:b")]);
        let local = arrange(
            full.clone(),
            Scope::Local,
            Some("note:a"),
            1,
            &GraphFilter::All,
            &[],
        );
        assert_eq!(local.graph.len(), 2);
        assert_eq!(local.pos.len(), 2);
        let none = arrange(full.clone(), Scope::Local, None, 1, &GraphFilter::All, &[]);
        assert!(none.graph.is_empty());
        let gone = arrange(
            full.clone(),
            Scope::Local,
            Some("note:zz"),
            1,
            &GraphFilter::All,
            &[],
        );
        assert!(gone.graph.is_empty());
        let all = arrange(
            full.clone(),
            Scope::Library,
            None,
            1,
            &GraphFilter::All,
            &[],
        );
        assert_eq!(all.graph.len(), 3);
        let book = arrange(
            full.clone(),
            Scope::Library,
            None,
            1,
            &GraphFilter::Book("book:a".into()),
            &[],
        );
        assert_eq!(book.graph.len(), 3);
        let other = arrange(
            full,
            Scope::Library,
            None,
            1,
            &GraphFilter::Tag("tag:x".into()),
            &[],
        );
        assert!(other.graph.is_empty());
    }

    proptest! {
        #![proptest_config(config(64))]

        /// A notebook's color is stable and differs between the themes.
        #[test]
        fn prop_book_colors_are_stable(id in "book:[a-zA-Z0-9]{1,12}") {
            prop_assert_eq!(book_color(&id, true), book_color(&id, true));
            prop_assert_ne!(book_color(&id, true), book_color(&id, false));
        }

        /// A notebook is inside every notebook on its way up the parents
        /// and inside no other, whatever the (possibly cyclic) parents.
        #[test]
        fn prop_a_notebook_is_inside_its_ancestors_only(
            parents in prop::collection::vec(prop::option::of(0usize..6), 6),
            start in 0usize..6,
            other in 0usize..6,
        ) {
            let name = |i: usize| format!("b{i}");
            let books: Vec<(String, Option<String>)> = parents
                .iter()
                .enumerate()
                .map(|(i, p)| (name(i), p.map(name)))
                .collect();
            let mut ancestors = vec![start];
            let mut at = start;
            while let Some(parent) = parents[at] {
                if ancestors.contains(&parent) { break; }
                ancestors.push(parent);
                at = parent;
            }
            prop_assert_eq!(
                in_notebook(&name(start), &name(other), &books),
                ancestors.contains(&other)
            );
        }

        /// A laid-out view has a position and a size for every node it shows.
        #[test]
        fn prop_an_arranged_graph_is_whole(graph in arb_graph(30), hops in 1u8..4, pick in 0usize..30) {
            let center = graph.nodes.get(pick).map(|n| n.id.clone());
            for scope in [Scope::Local, Scope::Library] {
                let arranged = arrange(graph.clone(), scope, center.as_deref(), hops, &GraphFilter::All, &[]);
                prop_assert_eq!(arranged.pos.len(), arranged.graph.len());
                prop_assert_eq!(arranged.radii.len(), arranged.graph.len());
                prop_assert!(arranged.pos.iter().all(|p| p[0].is_finite() && p[1].is_finite()));
                if scope == Scope::Local && let Some(center) = &center {
                    prop_assert!(arranged.graph.index_of(center).is_some());
                }
            }
        }
    }
}
