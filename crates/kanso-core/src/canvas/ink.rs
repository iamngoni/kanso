//! Ink geometry that needs no screen: the partial eraser's cut.
//!
//! Points are never rewritten. A stroke the eraser touches is tombstoned by
//! the caller and replaced by the pieces [`split_stroke`] returns: untouched
//! points are copied verbatim, and each cut lands exactly on the eraser's
//! edge, with one interpolated point (position, pressure and tilt, linearly)
//! on each side of it.

use super::model::{Point, Stroke};

/// The partial eraser's path in the ink node's own coordinates: a disc of
/// `radius` swept along `path`. A single sample is a plain disc.
#[derive(Clone, Debug, PartialEq)]
pub struct Eraser {
    pub path: Vec<(f64, f64)>,
    pub radius: f64,
}

/// What the eraser did to a stroke.
#[derive(Clone, Debug, PartialEq)]
pub enum Split {
    /// The eraser missed.
    Untouched,
    /// What is left of the stroke, in drawing order. Empty when none of it is.
    Pieces(Vec<Stroke>),
}

const EPS: f64 = 1e-9;

/// Cuts `eraser` out of `stroke`. Each piece takes the stroke's tool, colour,
/// width and extras, an ID from `fresh_id`, and version 0 (the caller stamps
/// the versions when it writes them). Pure.
pub fn split_stroke(
    stroke: &Stroke,
    eraser: &Eraser,
    mut fresh_id: impl FnMut() -> String,
) -> Split {
    let pts = &stroke.points;
    if pts.is_empty() || eraser.path.is_empty() || eraser.radius.is_nan() || eraser.radius <= 0.0 {
        return Split::Untouched;
    }
    let mut removed: Vec<(f64, f64)> = Vec::new();
    if pts.len() == 1 {
        if covers(eraser, pts[0].x, pts[0].y) {
            return Split::Pieces(Vec::new());
        }
        return Split::Untouched;
    }
    for (i, pair) in pts.windows(2).enumerate() {
        let (a, b) = ((pair[0].x, pair[0].y), (pair[1].x, pair[1].y));
        for (lo, hi) in segment_inside(a, b, eraser) {
            removed.push((i as f64 + lo, i as f64 + hi));
        }
    }
    if removed.is_empty() {
        return Split::Untouched;
    }
    removed.sort_by(|p, q| p.0.total_cmp(&q.0));
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for (lo, hi) in removed {
        match merged.last_mut() {
            Some(last) if lo <= last.1 + EPS => last.1 = last.1.max(hi),
            _ => merged.push((lo, hi)),
        }
    }
    let last = (pts.len() - 1) as f64;
    let mut kept: Vec<(f64, f64)> = Vec::new();
    let mut from = 0.0;
    for (lo, hi) in merged {
        if lo - from > EPS {
            kept.push((from, lo));
        }
        from = from.max(hi);
    }
    if last - from > EPS {
        kept.push((from, last));
    }
    let pieces = kept
        .into_iter()
        .map(|(a, b)| {
            let mut points = vec![at(pts, a)];
            let first = a.floor() as usize + 1;
            let end = b.ceil() as usize;
            points.extend_from_slice(&pts[first..end]);
            points.push(at(pts, b));
            Stroke {
                id: fresh_id(),
                tool: stroke.tool.clone(),
                color: stroke.color.clone(),
                width: stroke.width,
                points,
                deleted: false,
                kanso_version: 0,
                kanso_updated: 0,
                kanso_z: stroke.kanso_z,
                extra: stroke.extra.clone(),
            }
        })
        .collect();
    Split::Pieces(pieces)
}

/// Whether `(x, y)` is under the eraser.
pub fn covers(eraser: &Eraser, x: f64, y: f64) -> bool {
    let r2 = eraser.radius * eraser.radius;
    if let [only] = eraser.path.as_slice() {
        return dist2((x, y), *only) <= r2;
    }
    eraser
        .path
        .windows(2)
        .any(|w| dist2_to_segment((x, y), w[0], w[1]) <= r2)
}

/// The stroke's point at `u`, where the whole part is a point's index and the
/// rest runs along the segment to the next one. Exact points are returned as
/// they are.
fn at(pts: &[Point], u: f64) -> Point {
    let i = (u.floor() as usize).min(pts.len() - 1);
    let t = u - i as f64;
    if t <= 0.0 || i + 1 >= pts.len() {
        return pts[i];
    }
    let (p, q) = (pts[i], pts[i + 1]);
    let lerp = |a: f64, b: f64| a + (b - a) * t;
    Point {
        x: round(lerp(p.x, q.x), 100.0),
        y: round(lerp(p.y, q.y), 100.0),
        pressure: round(lerp(p.pressure, q.pressure), 1000.0),
        tilt: match (p.tilt, q.tilt) {
            (Some(a), Some(b)) => {
                Some((round(lerp(a.0, b.0), 1000.0), round(lerp(a.1, b.1), 1000.0)))
            }
            _ => None,
        },
    }
}

fn round(v: f64, scale: f64) -> f64 {
    (v * scale).round() / scale
}

fn dist2(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)
}

fn dist2_to_segment(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    if len2 <= EPS {
        return dist2(p, a);
    }
    let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0);
    dist2(p, (a.0 + t * dx, a.1 + t * dy))
}

/// The parts of segment `a`-`b`, as `t` ranges in 0..=1, that lie under the
/// eraser.
fn segment_inside(a: (f64, f64), b: (f64, f64), eraser: &Eraser) -> Vec<(f64, f64)> {
    let r = eraser.radius;
    if let [only] = eraser.path.as_slice() {
        return disc(a, b, *only, r).into_iter().collect();
    }
    eraser
        .path
        .windows(2)
        .filter_map(|w| stadium(a, b, w[0], w[1], r))
        .collect()
}

/// Segment `a`-`b` inside the disc at `c`.
fn disc(a: (f64, f64), b: (f64, f64), c: (f64, f64), r: f64) -> Option<(f64, f64)> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let big_a = dx * dx + dy * dy;
    let (ex, ey) = (a.0 - c.0, a.1 - c.1);
    let big_c = ex * ex + ey * ey - r * r;
    if big_a <= EPS {
        return (big_c <= 0.0).then_some((0.0, 1.0));
    }
    let big_b = dx * ex + dy * ey;
    let disc = big_b * big_b - big_a * big_c;
    if disc < 0.0 {
        return None;
    }
    let root = disc.sqrt();
    let (lo, hi) = (
        ((-big_b - root) / big_a).max(0.0),
        ((-big_b + root) / big_a).min(1.0),
    );
    (lo <= hi).then_some((lo, hi))
}

/// Segment `a`-`b` inside the capsule (a disc swept from `c` to `d`). A
/// capsule is convex, so the answer is one range: the hull of what lies in
/// its two end discs and its rectangle.
fn stadium(
    a: (f64, f64),
    b: (f64, f64),
    c: (f64, f64),
    d: (f64, f64),
    r: f64,
) -> Option<(f64, f64)> {
    let mut parts: Vec<(f64, f64)> = Vec::new();
    parts.extend(disc(a, b, c, r));
    parts.extend(disc(a, b, d, r));
    let (ux, uy) = (d.0 - c.0, d.1 - c.1);
    let len = (ux * ux + uy * uy).sqrt();
    if len > EPS {
        let (ux, uy) = (ux / len, uy / len);
        // Along the capsule (s, 0..len) and across it (w, -r..r); both are
        // linear in t, so each bound clips the range.
        let s0 = (a.0 - c.0) * ux + (a.1 - c.1) * uy;
        let s1 = (b.0 - c.0) * ux + (b.1 - c.1) * uy;
        let w0 = -(a.0 - c.0) * uy + (a.1 - c.1) * ux;
        let w1 = -(b.0 - c.0) * uy + (b.1 - c.1) * ux;
        let mut range = Some((0.0, 1.0));
        for (v0, v1, lo, hi) in [(s0, s1, 0.0, len), (w0, w1, -r, r)] {
            range = range.and_then(|rg| clip(rg, v0, v1, lo, hi));
        }
        parts.extend(range);
    }
    let lo = parts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let hi = parts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    (lo <= hi).then_some((lo, hi))
}

/// The part of `range` where `v0 + (v1 - v0) * t` stays within `lo..=hi`.
fn clip(range: (f64, f64), v0: f64, v1: f64, lo: f64, hi: f64) -> Option<(f64, f64)> {
    let dv = v1 - v0;
    let (mut t0, mut t1) = range;
    if dv.abs() <= EPS {
        return (lo..=hi).contains(&v0).then_some((t0, t1));
    }
    let (p, q) = ((lo - v0) / dv, (hi - v0) / dv);
    t0 = t0.max(p.min(q));
    t1 = t1.min(p.max(q));
    (t0 <= t1).then_some((t0, t1))
}
