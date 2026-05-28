//! Deterministic stroke geometry — shared by every platform so a sketch looks
//! the same on macOS, iPad, and Android.

use crate::format::Point;

/// Map captured pen pressure to a rendered stroke width.
///
/// Keeps a floor so zero-pressure samples still draw, and scales up to the full
/// base width at maximum pressure.
pub fn pressure_to_width(base_width: f32, pressure: f32) -> f32 {
    let p = pressure.clamp(0.0, 1.0);
    base_width * (0.35 + 0.65 * p)
}

/// Catmull-Rom smoothing: produce a denser polyline that passes through every
/// input point, rounding the corners that raw sampling leaves jagged.
///
/// `samples_per_segment` controls smoothness vs. cost. Inputs shorter than 3
/// points are returned unchanged.
pub fn smooth(points: &[Point], samples_per_segment: usize) -> Vec<Point> {
    if points.len() < 3 || samples_per_segment == 0 {
        return points.to_vec();
    }

    let n = points.len();
    let mut out = Vec::with_capacity(n * samples_per_segment + 1);

    for i in 0..n - 1 {
        let p0 = points[i.saturating_sub(1)];
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = points[(i + 2).min(n - 1)];

        for s in 0..samples_per_segment {
            let t = s as f32 / samples_per_segment as f32;
            out.push(catmull_rom(p0, p1, p2, p3, t));
        }
    }
    out.push(points[n - 1]);
    out
}

fn catmull_rom(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
    let t2 = t * t;
    let t3 = t2 * t;
    let interp = |a: f32, b: f32, c: f32, d: f32| {
        0.5 * ((2.0 * b)
            + (-a + c) * t
            + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
            + (-a + 3.0 * b - 3.0 * c + d) * t3)
    };

    Point {
        x: interp(p0.x, p1.x, p2.x, p3.x),
        y: interp(p0.y, p1.y, p2.y, p3.y),
        // Pressure and time interpolate linearly along the segment.
        pressure: p1.pressure + (p2.pressure - p1.pressure) * t,
        tilt: p1.tilt,
        t: p1.t + (p2.t - p1.t) * t,
    }
}

/// Axis-aligned bounds `(min_x, min_y, max_x, max_y)` of a set of points.
pub fn bounds<'a>(points: impl Iterator<Item = &'a Point>) -> Option<(f32, f32, f32, f32)> {
    let mut acc: Option<(f32, f32, f32, f32)> = None;
    for p in points {
        acc = Some(match acc {
            None => (p.x, p.y, p.x, p.y),
            Some((minx, miny, maxx, maxy)) => {
                (minx.min(p.x), miny.min(p.y), maxx.max(p.x), maxy.max(p.y))
            }
        });
    }
    acc
}
