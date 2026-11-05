//! Characters to lines, lines to blocks, blocks to reading order.
//!
//! Text is grouped in an "upright" frame: the page turned so the text of
//! the group runs left to right with lines going down. A group is all the
//! characters that run the same way on the display, so a rotated page and a
//! sideways label on an upright page are handled by the same code.
//!
//! Left-to-right scripts are assumed. Every threshold is a fraction of the
//! group's median character height (or, for the fallback word break, of a
//! line's own character pitch), so the layout does not depend on the page's
//! size or zoom.
//!
//! Words break where PDFium put a space ([`TextChar::space_before`]). No
//! single gap width tells a space from the gap inside a word across fonts:
//! measured on real output, Helvetica's widest in-word gap is 0.37 of the
//! glyph height and its narrowest word gap 0.38, Times 0.27 and 0.375,
//! Courier 0.63 and 1.1. Gaps are measured only on a page where PDFium
//! reported no space at all.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use super::model::{Block, Glyph, Line, PageLayout, PageText, Sep};
use crate::model::Quad;

/// A gap wider than this (times the median height) inside a line splits it
/// into two pieces: a column gutter or a table cell edge.
const SPLIT_GAP: f64 = 1.5;
/// Fallback only, on a page where PDFium reported no space: a gap wider
/// than this (times the line's median character pitch, the distance from
/// one character's left edge to the next one's) reads as a space. Measured
/// on real output, in those units: Helvetica's widest gap inside a word is
/// 0.38 and its narrowest between words 0.44; Times 0.28 and 0.38.
const SPACE_GAP: f64 = 0.41;
/// The same on a line set in a fixed-pitch font (Courier: 0.55 inside a
/// word, 1.04 between words), where a narrow letter leaves a wide gap but a
/// space skips a whole cell.
const FIXED_PITCH_SPACE_GAP: f64 = 0.8;
/// A line is fixed-pitch when this share of its advances are within
/// [`FIXED_PITCH_SLACK`] of the median one.
const FIXED_PITCH_SHARE: f64 = 0.7;
const FIXED_PITCH_SLACK: f64 = 0.03;
/// Lines closer than this (vertical gap, times the height) share a block.
const BLOCK_GAP: f64 = 0.5;
/// A vertical cut between columns needs a gutter this wide.
const COLUMN_GAP: f64 = 1.0;

type Rect = [f64; 4];

/// A character in the upright frame.
#[derive(Clone, Copy)]
struct Item {
    ch: char,
    /// PDFium put a space just before this character.
    space_before: bool,
    /// The displayed quad, kept to build the output.
    quad: Quad,
    rect: Rect,
}

/// One piece of a line: items left to right.
struct Segment {
    items: Vec<usize>,
    rect: Rect,
    height: f64,
    /// The fallback space width: the gap past which two neighbours read as
    /// two words. Scaled by the line's average advance (the median distance
    /// between the left edges of neighbouring characters), not by its
    /// height, which is a different fraction of the advance in every font.
    space_gap: f64,
}

/// A character about to be laid out.
struct Piece {
    ch: char,
    quad: Quad,
    space_before: bool,
}

fn cmp(a: f64, b: f64) -> Ordering {
    a.total_cmp(&b)
}

/// The display point `(x, y)` on a `w` x `h` page, in the frame where text
/// that runs `d` quarter turns clockwise reads left to right.
fn to_upright(d: u8, w: f64, h: f64, x: f64, y: f64) -> (f64, f64) {
    match d % 4 {
        0 => (x, y),
        1 => (y, w - x),
        2 => (w - x, h - y),
        _ => (h - y, x),
    }
}

fn upright_rect(d: u8, w: f64, h: f64, q: &Quad) -> Rect {
    let [x0, y0, x1, y1] = q.0;
    let (ax, ay) = to_upright(d, w, h, x0 * w, y0 * h);
    let (bx, by) = to_upright(d, w, h, x1 * w, y1 * h);
    [ax.min(bx), ay.min(by), ax.max(bx), ay.max(by)]
}

fn union(a: Rect, b: Rect) -> Rect {
    [
        a[0].min(b[0]),
        a[1].min(b[1]),
        a[2].max(b[2]),
        a[3].max(b[3]),
    ]
}

fn quad_union(a: &Quad, b: &Quad) -> Quad {
    let u = union(a.0, b.0);
    Quad(u)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| cmp(*a, *b));
    match v.len() {
        0 => 1.0,
        n => v[n / 2],
    }
}

/// Lay out one page. Characters may come in any order; the same set of
/// characters always gives the same layout.
pub fn layout_page(input: &PageText) -> PageLayout {
    let (w, h) = (
        if input.width.is_finite() && input.width > 0.0 {
            input.width
        } else {
            1.0
        },
        if input.height.is_finite() && input.height > 0.0 {
            input.height
        } else {
            1.0
        },
    );
    let page_turns = ((input.rotation % 360) / 90) as u8;

    // Characters that run the same way on the display form a group.
    let mut groups: BTreeMap<u8, Vec<Piece>> = BTreeMap::new();
    for c in &input.chars {
        if c.ch.is_whitespace() || c.ch.is_control() {
            continue;
        }
        let d = (page_turns + c.turns % 4) % 4;
        let [a, b, c2, e] = c.quad.0;
        groups.entry(d).or_default().push(Piece {
            ch: c.ch,
            quad: Quad::new(a, b, c2, e),
            space_before: c.space_before,
        });
    }
    // PDFium's spaces are trusted when it reported any on the page.
    let reported = input.chars.iter().any(|c| c.space_before);
    // The direction most characters run in reads first.
    let mut order: Vec<(u8, Vec<Piece>)> = groups.into_iter().collect();
    order.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(&b.0)));

    let mut out = PageLayout {
        page: input.page,
        width: input.width,
        height: input.height,
        glyphs: Vec::new(),
        lines: Vec::new(),
        blocks: Vec::new(),
    };
    for (d, chars) in order {
        layout_group(&mut out, d, (w, h), reported, chars);
    }
    out
}

fn layout_group(
    out: &mut PageLayout,
    d: u8,
    (w, h): (f64, f64),
    reported: bool,
    chars: Vec<Piece>,
) {
    let mut items: Vec<Item> = chars
        .into_iter()
        .map(|p| Item {
            ch: p.ch,
            space_before: p.space_before,
            quad: p.quad,
            rect: upright_rect(d, w, h, &p.quad),
        })
        .collect();
    // A total order, so the input order never matters.
    items.sort_by(|a, b| {
        cmp(a.rect[1] + a.rect[3], b.rect[1] + b.rect[3])
            .then(cmp(a.rect[0], b.rect[0]))
            .then(cmp(a.rect[1], b.rect[1]))
            .then(cmp(a.rect[2], b.rect[2]))
            .then(cmp(a.rect[3], b.rect[3]))
            .then(a.ch.cmp(&b.ch))
            .then(a.space_before.cmp(&b.space_before))
    });
    let med = median(items.iter().map(|i| i.rect[3] - i.rect[1]).collect()).max(1e-3);

    let segments = segments(&items, med);
    let blocks = blocks(&segments);
    let order = reading_order(&blocks, med);

    for &bi in &order {
        let block = out.blocks.len() as u32;
        let first_glyph = out.glyphs.len();
        for (si_pos, &si) in blocks[bi].segs.iter().enumerate() {
            let seg = &segments[si];
            let line = out.lines.len() as u32;
            let line_start = out.glyphs.len();
            let mut band: Option<Quad> = None;
            let mut reach = f64::NEG_INFINITY;
            for (k, &ii) in seg.items.iter().enumerate() {
                let it = &items[ii];
                let sep = if k == 0 {
                    if out.glyphs.is_empty() {
                        Sep::None
                    } else if si_pos == 0 {
                        Sep::Paragraph
                    } else {
                        Sep::Line
                    }
                } else if if reported {
                    it.space_before
                } else {
                    it.rect[0] - reach > seg.space_gap
                } {
                    Sep::Space
                } else {
                    Sep::None
                };
                reach = if k == 0 {
                    it.rect[2]
                } else {
                    reach.max(it.rect[2])
                };
                band = Some(match band {
                    None => it.quad,
                    Some(b) => quad_union(&b, &it.quad),
                });
                out.glyphs.push(Glyph {
                    ch: it.ch,
                    quad: it.quad,
                    sep,
                    line,
                    block,
                });
            }
            out.lines.push(Line {
                glyphs: line_start..out.glyphs.len(),
                band: band.unwrap_or(Quad([0.0; 4])),
                turns: d,
            });
        }
        out.blocks.push(Block {
            glyphs: first_glyph..out.glyphs.len(),
        });
    }
}

/// Characters to line pieces.
fn segments(items: &[Item], med: f64) -> Vec<Segment> {
    // Cluster by vertical position. Items are sorted by their centre.
    struct Cluster {
        sum_cy: f64,
        sum_h: f64,
        n: f64,
        members: Vec<usize>,
    }
    let mut clusters: Vec<Cluster> = Vec::new();
    for (i, it) in items.iter().enumerate() {
        let hh = (it.rect[3] - it.rect[1]).max(0.05 * med);
        let cy = (it.rect[1] + it.rect[3]) / 2.0;
        let mut best: Option<(usize, f64)> = None;
        for (ci, c) in clusters.iter().enumerate().rev().take(64) {
            let core_h = c.sum_h / c.n;
            let core_cy = c.sum_cy / c.n;
            let lo = (cy - hh / 2.0).max(core_cy - core_h / 2.0);
            let hi = (cy + hh / 2.0).min(core_cy + core_h / 2.0);
            let overlap = hi - lo;
            if overlap >= 0.5 * hh.min(core_h) - 1e-9 && best.is_none_or(|(_, o)| overlap > o) {
                best = Some((ci, overlap));
            }
        }
        match best {
            Some((ci, _)) => {
                let c = &mut clusters[ci];
                c.sum_cy += cy;
                c.sum_h += hh;
                c.n += 1.0;
                c.members.push(i);
            }
            None => clusters.push(Cluster {
                sum_cy: cy,
                sum_h: hh,
                n: 1.0,
                members: vec![i],
            }),
        }
    }

    // Split each cluster at wide horizontal gaps.
    let mut segs = Vec::new();
    for c in clusters {
        let mut m = c.members;
        m.sort_by(|&a, &b| {
            cmp(items[a].rect[0], items[b].rect[0])
                .then(cmp(items[a].rect[2], items[b].rect[2]))
                .then(cmp(items[a].rect[1], items[b].rect[1]))
                .then(cmp(items[a].rect[3], items[b].rect[3]))
                .then(items[a].ch.cmp(&items[b].ch))
                .then(items[a].space_before.cmp(&items[b].space_before))
        });
        let mut cur: Vec<usize> = Vec::new();
        let mut reach = f64::NEG_INFINITY;
        for i in m {
            if !cur.is_empty() && items[i].rect[0] - reach > SPLIT_GAP * med {
                segs.push(make_segment(items, std::mem::take(&mut cur)));
            }
            reach = if cur.is_empty() {
                items[i].rect[2]
            } else {
                reach.max(items[i].rect[2])
            };
            cur.push(i);
        }
        if !cur.is_empty() {
            segs.push(make_segment(items, cur));
        }
    }
    segs
}

fn make_segment(items: &[Item], idx: Vec<usize>) -> Segment {
    let rect = idx[1..]
        .iter()
        .fold(items[idx[0]].rect, |r, &i| union(r, items[i].rect));
    let height = median(
        idx.iter()
            .map(|&i| items[i].rect[3] - items[i].rect[1])
            .collect(),
    )
    .max(1e-3);
    // Items come left to right. A repeated left edge (two boxes stacked on
    // one another) is no advance. With no advance to measure, half the
    // height stands in for it.
    let advances: Vec<f64> = idx
        .windows(2)
        .map(|p| items[p[1]].rect[0] - items[p[0]].rect[0])
        .filter(|d| *d > 1e-6)
        .collect();
    let (pitch, fixed) = if advances.is_empty() {
        (0.5 * height, false)
    } else {
        let pitch = median(advances.clone());
        let same = advances
            .iter()
            .filter(|a| (**a - pitch).abs() <= FIXED_PITCH_SLACK * pitch)
            .count();
        (
            pitch,
            advances.len() >= 3 && same as f64 >= FIXED_PITCH_SHARE * advances.len() as f64,
        )
    };
    let space_gap = pitch
        * if fixed {
            FIXED_PITCH_SPACE_GAP
        } else {
            SPACE_GAP
        };
    Segment {
        items: idx,
        rect,
        height,
        space_gap,
    }
}

struct BlockSegs {
    segs: Vec<usize>,
    rect: Rect,
}

/// Line pieces to blocks: pieces stacked closely and lined up share one.
fn blocks(segments: &[Segment]) -> Vec<BlockSegs> {
    let mut order: Vec<usize> = (0..segments.len()).collect();
    order.sort_by(|&a, &b| {
        cmp(segments[a].rect[1], segments[b].rect[1])
            .then(cmp(segments[a].rect[0], segments[b].rect[0]))
            .then(cmp(segments[a].rect[2], segments[b].rect[2]))
            .then(cmp(segments[a].rect[3], segments[b].rect[3]))
            .then(segments[a].items.len().cmp(&segments[b].items.len()))
    });
    let mut blocks: Vec<BlockSegs> = Vec::new();
    for si in order {
        let s = &segments[si];
        let mut best: Option<(usize, f64)> = None;
        for (bi, b) in blocks.iter().enumerate().rev().take(64) {
            let last = &segments[*b.segs.last().expect("a block has a line")];
            let hh = (last.height + s.height) / 2.0;
            let ratio = last.height.max(s.height) / last.height.min(s.height);
            let gap = s.rect[1] - last.rect[3];
            let overlap = s.rect[2].min(last.rect[2]) - s.rect[0].max(last.rect[0]);
            let narrow = (s.rect[2] - s.rect[0]).min(last.rect[2] - last.rect[0]);
            if ratio <= 1.5
                && gap >= -0.25 * hh
                && gap <= BLOCK_GAP * hh
                && overlap >= 0.5 * narrow
                && best.is_none_or(|(_, g)| gap < g)
            {
                best = Some((bi, gap));
            }
        }
        match best {
            Some((bi, _)) => {
                let b = &mut blocks[bi];
                b.rect = union(b.rect, s.rect);
                b.segs.push(si);
            }
            None => blocks.push(BlockSegs {
                segs: vec![si],
                rect: s.rect,
            }),
        }
    }
    blocks
}

/// Blocks in reading order: XY-cut. Columns first (a vertical gutter that
/// the neighbours share some height across), then rows.
fn reading_order(blocks: &[BlockSegs], med: f64) -> Vec<usize> {
    let ids: Vec<usize> = (0..blocks.len()).collect();
    let mut out = Vec::with_capacity(ids.len());
    cut(&ids, blocks, COLUMN_GAP * med, &mut out);
    out
}

/// Groups of `ids` separated by a gap of more than `min_gap` along `axis`
/// (0 = x, 1 = y), in order along that axis.
fn split_axis(ids: &[usize], blocks: &[BlockSegs], axis: usize, min_gap: f64) -> Vec<Vec<usize>> {
    let mut v = ids.to_vec();
    v.sort_by(|&a, &b| {
        cmp(blocks[a].rect[axis], blocks[b].rect[axis])
            .then(cmp(blocks[a].rect[axis + 2], blocks[b].rect[axis + 2]))
            .then(a.cmp(&b))
    });
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut reach = f64::NEG_INFINITY;
    for i in v {
        let r = &blocks[i].rect;
        if groups.is_empty() || r[axis] - reach > min_gap {
            groups.push(Vec::new());
            reach = r[axis + 2];
        } else {
            reach = reach.max(r[axis + 2]);
        }
        groups.last_mut().expect("just pushed").push(i);
    }
    groups
}

fn extent(group: &[usize], blocks: &[BlockSegs], axis: usize) -> (f64, f64) {
    group
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &i| {
            (
                lo.min(blocks[i].rect[axis]),
                hi.max(blocks[i].rect[axis + 2]),
            )
        })
}

fn cut(ids: &[usize], blocks: &[BlockSegs], col_gap: f64, out: &mut Vec<usize>) {
    if ids.len() <= 1 {
        out.extend_from_slice(ids);
        return;
    }
    let cols = split_axis(ids, blocks, 0, col_gap);
    if cols.len() > 1 && columns_share_height(&cols, blocks) {
        for g in &cols {
            cut(g, blocks, col_gap, out);
        }
        return;
    }
    let rows = split_axis(ids, blocks, 1, 0.0);
    if rows.len() > 1 {
        for g in &rows {
            cut(g, blocks, col_gap, out);
        }
        return;
    }
    let mut v = ids.to_vec();
    v.sort_by(|&a, &b| {
        cmp(blocks[a].rect[1], blocks[b].rect[1])
            .then(cmp(blocks[a].rect[0], blocks[b].rect[0]))
            .then(a.cmp(&b))
    });
    out.extend(v);
}

/// A column cut counts only when neighbouring columns sit side by side for
/// at least half of the shorter one. A short note in a corner is not a
/// column.
fn columns_share_height(cols: &[Vec<usize>], blocks: &[BlockSegs]) -> bool {
    cols.windows(2).all(|p| {
        let (a0, a1) = extent(&p[0], blocks, 1);
        let (b0, b1) = extent(&p[1], blocks, 1);
        let overlap = a1.min(b1) - a0.max(b0);
        overlap >= 0.5 * (a1 - a0).min(b1 - b0)
    })
}
