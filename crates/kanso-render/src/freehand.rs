//! A port of [perfect-freehand](https://github.com/steveruizok/perfect-freehand)
//! (MIT, Steve Ruiz): pen samples in, the outline of a stroke out. Pressure
//! widens the line; without real pressure (a mouse, a finger) it is
//! simulated from speed. Pure, with no screen in it, so the desktop view, the
//! phone and the SVG renderer draw the same stroke.

use std::f64::consts::PI;

/// A little over pi, as perfect-freehand uses, so a half turn closes cleanly.
const FIXED_PI: f64 = PI + 0.0001;
/// How fast simulated pressure follows speed.
const RATE_OF_PRESSURE_CHANGE: f64 = 0.275;
/// The most samples a stroke is outlined from.
pub const MAX_SAMPLES: usize = 40_000;

/// One input sample: position and pressure (0 to 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub x: f64,
    pub y: f64,
    pub pressure: f64,
}

impl Sample {
    pub fn new(x: f64, y: f64, pressure: f64) -> Self {
        Sample { x, y, pressure }
    }
}

/// How a stroke is shaped. `size` is the widest the line gets, in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Options {
    pub size: f64,
    pub thinning: f64,
    pub smoothing: f64,
    pub streamline: f64,
    pub simulate_pressure: bool,
    /// A taper at the start and end, in points of length. 0 is none.
    pub taper_start: f64,
    pub taper_end: f64,
    /// Whether the stroke is finished: the last point is kept as it is.
    pub complete: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            size: 16.0,
            thinning: 0.5,
            smoothing: 0.5,
            streamline: 0.5,
            simulate_pressure: true,
            taper_start: 0.0,
            taper_end: 0.0,
            complete: false,
        }
    }
}

type V = [f64; 2];

fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1]]
}
fn add(a: V, b: V) -> V {
    [a[0] + b[0], a[1] + b[1]]
}
fn mul(a: V, k: f64) -> V {
    [a[0] * k, a[1] * k]
}
fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn per(a: V) -> V {
    [a[1], -a[0]]
}
fn dist2(a: V, b: V) -> f64 {
    let d = sub(a, b);
    dot(d, d)
}
fn dist(a: V, b: V) -> f64 {
    dist2(a, b).sqrt()
}
fn uni(a: V) -> V {
    let len = dot(a, a).sqrt();
    if len == 0.0 {
        [0.0, 0.0]
    } else {
        mul(a, 1.0 / len)
    }
}
fn lerp(a: V, b: V, t: f64) -> V {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}
fn rot_around(a: V, c: V, r: f64) -> V {
    let (s, co) = r.sin_cos();
    let (px, py) = (a[0] - c[0], a[1] - c[1]);
    [px * co - py * s + c[0], px * s + py * co + c[1]]
}
fn ease_out_quad(t: f64) -> f64 {
    t * (2.0 - t)
}

/// A sample after streamlining: where the line passes and how it was moving.
#[derive(Clone, Copy)]
struct Stroked {
    point: V,
    pressure: f64,
    /// Unit vector from this point back to the previous one.
    vector: V,
    distance: f64,
    running: f64,
}

/// Streamlines `samples`: each point is pulled toward the line, short wiggles
/// before the stroke is `size` long are dropped. Pure.
fn stroke_points(samples: &[Sample], opts: &Options) -> Vec<Stroked> {
    let t = 0.15 + (1.0 - opts.streamline.clamp(0.0, 1.0)) * 0.85;
    let mut pts: Vec<(V, f64)> = samples
        .iter()
        .filter(|s| s.x.is_finite() && s.y.is_finite())
        .take(MAX_SAMPLES)
        .map(|s| {
            let p = if s.pressure.is_finite() {
                s.pressure.clamp(0.0, 1.0)
            } else {
                0.5
            };
            ([s.x, s.y], p)
        })
        .collect();
    if pts.is_empty() {
        return Vec::new();
    }
    if pts.len() == 1 {
        let (p, pr) = pts[0];
        pts.push((add(p, [1.0, 1.0]), pr));
    }
    let mut out = vec![Stroked {
        point: pts[0].0,
        pressure: pts[0].1,
        vector: [1.0, 1.0],
        distance: 0.0,
        running: 0.0,
    }];
    let (mut reached, mut running) = (false, 0.0);
    let mut prev = out[0];
    let max = pts.len() - 1;
    for (i, &(raw, pressure)) in pts.iter().enumerate().skip(1) {
        let point = if opts.complete && i == max {
            raw
        } else {
            lerp(prev.point, raw, t)
        };
        if prev.point == point {
            continue;
        }
        let distance = dist(point, prev.point);
        running += distance;
        if i < max && !reached {
            if running < opts.size {
                continue;
            }
            reached = true;
        }
        prev = Stroked {
            point,
            pressure,
            vector: uni(sub(prev.point, point)),
            distance,
            running,
        };
        out.push(prev);
    }
    if out.len() > 1 {
        out[0].vector = out[1].vector;
    } else {
        out[0].vector = [0.0, 0.0];
    }
    out
}

fn radius(size: f64, thinning: f64, pressure: f64) -> f64 {
    size * (0.5 - thinning * (0.5 - pressure))
}

/// The outline of the stroke through `samples`, as a closed polygon. Empty
/// for no samples. Pure.
pub fn outline(samples: &[Sample], opts: &Options) -> Vec<[f64; 2]> {
    let pts = stroke_points(samples, opts);
    if pts.is_empty() {
        return Vec::new();
    }
    let size = opts.size.max(0.01);
    let n = pts.len();
    let total = pts[n - 1].running;
    let (taper_start, taper_end) = (opts.taper_start.max(0.0), opts.taper_end.max(0.0));
    let min_dist = (size * opts.smoothing) * (size * opts.smoothing);

    let mut left: Vec<V> = Vec::new();
    let mut right: Vec<V> = Vec::new();

    let mut prev_pressure = pts.iter().take(10).fold(pts[0].pressure, |acc, p| {
        let mut pressure = p.pressure;
        if opts.simulate_pressure {
            let sp = (p.distance / size).min(1.0);
            let rp = (1.0 - sp).min(1.0);
            pressure = (acc + (rp - acc) * (sp * RATE_OF_PRESSURE_CHANGE)).min(1.0);
        }
        (acc + pressure) / 2.0
    });

    let mut rad = radius(size, opts.thinning, pts[n - 1].pressure);
    let mut first_radius: Option<f64> = None;
    let mut prev_vector = pts[0].vector;
    let (mut pl, mut pr) = (pts[0].point, pts[0].point);
    let mut prev_sharp = false;

    for i in 0..n {
        let p = pts[i];
        let mut pressure = p.pressure;
        if i < n - 1 && total - p.running < 3.0 {
            continue;
        }
        if opts.thinning != 0.0 {
            if opts.simulate_pressure {
                let sp = (p.distance / size).min(1.0);
                let rp = (1.0 - sp).min(1.0);
                pressure =
                    (prev_pressure + (rp - prev_pressure) * (sp * RATE_OF_PRESSURE_CHANGE)).min(1.0);
            }
            rad = radius(size, opts.thinning, pressure);
        } else {
            rad = size / 2.0;
        }
        first_radius.get_or_insert(rad);
        let ts = if taper_start > 0.0 && p.running < taper_start {
            ease_out_quad(p.running / taper_start)
        } else {
            1.0
        };
        let te = if taper_end > 0.0 && total - p.running < taper_end {
            ease_out_quad((total - p.running) / taper_end)
        } else {
            1.0
        };
        rad = (rad * ts.min(te)).max(0.01);

        let next_vector = if i < n - 1 { pts[i + 1].vector } else { p.vector };
        let next_dpr = if i < n - 1 {
            dot(p.vector, next_vector)
        } else {
            1.0
        };
        let prev_dpr = dot(p.vector, prev_vector);
        let sharp = prev_dpr < 0.0 && !prev_sharp;
        let next_sharp = next_dpr < 0.0;
        if sharp || next_sharp {
            let offset = mul(per(prev_vector), rad);
            for step in 0..=13 {
                let k = step as f64 / 13.0;
                left.push(rot_around(sub(p.point, offset), p.point, FIXED_PI * k));
                right.push(rot_around(add(p.point, offset), p.point, -FIXED_PI * k));
            }
            pl = *left.last().unwrap();
            pr = *right.last().unwrap();
            if next_sharp {
                prev_sharp = true;
            }
            continue;
        }
        prev_sharp = false;
        if i == n - 1 {
            let offset = mul(per(p.vector), rad);
            left.push(sub(p.point, offset));
            right.push(add(p.point, offset));
            continue;
        }
        let offset = mul(per(lerp(next_vector, p.vector, next_dpr)), rad);
        let tl = sub(p.point, offset);
        if i <= 1 || dist2(pl, tl) > min_dist {
            left.push(tl);
            pl = tl;
        }
        let tr = add(p.point, offset);
        if i <= 1 || dist2(pr, tr) > min_dist {
            right.push(tr);
            pr = tr;
        }
        prev_pressure = pressure;
        prev_vector = p.vector;
    }

    let first = pts[0].point;
    let last = if n > 1 {
        pts[n - 1].point
    } else {
        add(pts[0].point, [1.0, 1.0])
    };

    if n == 1 && (!(taper_start > 0.0 || taper_end > 0.0) || opts.complete) {
        // A dot: a full circle.
        let r = first_radius.unwrap_or(rad);
        let start = add(first, mul(uni(per(sub(first, last))), -r));
        return (1..=13)
            .map(|s| rot_around(start, first, FIXED_PI * 2.0 * (s as f64 / 13.0)))
            .collect();
    }

    let mut start_cap: Vec<V> = Vec::new();
    let mut end_cap: Vec<V> = Vec::new();
    if taper_start == 0.0
        && let Some(&from) = right.first()
    {
        // Round the start: from the right edge, round the back, to the left.
        for s in 1..=13 {
            let k = s as f64 / 13.0;
            start_cap.push(rot_around(from, first, FIXED_PI * k));
        }
    }
    if taper_end > 0.0 {
        end_cap.push(last);
    } else {
        let direction = per(mul(pts[n - 1].vector, -1.0));
        let start = add(last, mul(direction, rad));
        for s in 1..29 {
            end_cap.push(rot_around(start, last, FIXED_PI * (s as f64 / 29.0)));
        }
    }

    right.reverse();
    let mut out = left;
    out.extend(end_cap);
    out.extend(right);
    out.extend(start_cap);
    out
}

/// A closed SVG path through `polygon`, its corners rounded with quadratic
/// curves through the midpoints of its edges (perfect-freehand's own way).
/// Empty for fewer than four points.
pub fn path_data(polygon: &[[f64; 2]]) -> String {
    use std::fmt::Write as _;
    let n = polygon.len();
    if n < 4 {
        return String::new();
    }
    let f = |v: f64| {
        let r = (v * 100.0).round() / 100.0;
        if r == 0.0 { 0.0 } else { r }
    };
    let mid = |a: V, b: V| [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
    let m = mid(polygon[0], polygon[1]);
    let mut d = format!(
        "M{} {}Q{} {} {} {}T",
        f(m[0]),
        f(m[1]),
        f(polygon[0][0]),
        f(polygon[0][1]),
        f(m[0]),
        f(m[1])
    );
    for i in 1..=n {
        let c = mid(polygon[i % n], polygon[(i + 1) % n]);
        let _ = write!(d, "{} {} ", f(c[0]), f(c[1]));
    }
    d.push('Z');
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn line(n: usize, pressure: f64) -> Vec<Sample> {
        (0..n)
            .map(|i| Sample::new(i as f64 * 5.0, (i as f64 * 0.7).sin() * 4.0, pressure))
            .collect()
    }

    #[test]
    fn empty_input_has_no_outline() {
        assert!(outline(&[], &Options::default()).is_empty());
    }

    #[test]
    fn a_dot_is_a_circle() {
        let opts = Options {
            size: 8.0,
            ..Options::default()
        };
        let o = outline(&[Sample::new(10.0, 10.0, 0.5)], &opts);
        assert_eq!(o.len(), 13);
        for p in o {
            let d = dist(p, [10.0, 10.0]);
            assert!(d > 0.5 && d < 8.0, "{d}");
        }
    }

    #[test]
    fn a_line_outline_stays_near_the_line() {
        let samples = line(40, 0.5);
        let opts = Options {
            size: 6.0,
            simulate_pressure: false,
            ..Options::default()
        };
        let o = outline(&samples, &opts);
        assert!(o.len() > 20);
        for p in &o {
            let near = samples
                .iter()
                .map(|s| dist(*p, [s.x, s.y]))
                .fold(f64::MAX, f64::min);
            assert!(near <= 6.0 + 1.0, "{near}");
        }
    }

    #[test]
    fn more_pressure_is_a_wider_line() {
        let width = |pressure: f64| {
            let opts = Options {
                size: 10.0,
                simulate_pressure: false,
                ..Options::default()
            };
            let o = outline(&line(30, pressure), &opts);
            let ys = o.iter().map(|p| p[1]);
            ys.clone().fold(f64::MIN, f64::max) - ys.fold(f64::MAX, f64::min)
        };
        assert!(width(1.0) > width(0.1));
    }

    #[test]
    fn caps_round_the_ends() {
        let samples: Vec<_> = (0..20)
            .map(|i| Sample::new(i as f64 * 4.0, 0.0, 0.5))
            .collect();
        let opts = Options {
            size: 8.0,
            thinning: 0.0,
            simulate_pressure: false,
            complete: true,
            ..Options::default()
        };
        let o = outline(&samples, &opts);
        let min_x = o.iter().map(|p| p[0]).fold(f64::MAX, f64::min);
        let max_x = o.iter().map(|p| p[0]).fold(f64::MIN, f64::max);
        // The round caps reach about a radius (4) past the first and last points.
        assert!(min_x < -2.0, "{min_x}");
        assert!(max_x > 76.0 + 2.0, "{max_x}");
    }

    #[test]
    fn path_is_closed_and_numeric() {
        let o = outline(&line(20, 0.5), &Options::default());
        let d = path_data(&o);
        assert!(d.starts_with('M') && d.ends_with('Z'));
        assert!(!d.contains("NaN"));
        assert_eq!(path_data(&[[0.0, 0.0]]), "");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Invariant: any finite samples outline to finite numbers, and a
        /// non-empty input never outlines to nothing.
        #[test]
        fn prop_outline_is_finite(
            raw in prop::collection::vec((-500f64..500., -500f64..500., 0f64..=1.), 1..60),
            size in 0.5f64..40.,
            simulate in any::<bool>(),
            complete in any::<bool>(),
        ) {
            let samples: Vec<_> = raw.into_iter().map(|(x, y, p)| Sample::new(x, y, p)).collect();
            let o = outline(&samples, &Options { size, simulate_pressure: simulate, complete, ..Options::default() });
            prop_assert!(!o.is_empty());
            prop_assert!(o.iter().all(|p| p[0].is_finite() && p[1].is_finite()));
            let d = path_data(&o);
            prop_assert!(!d.contains("NaN") && !d.contains("inf"));
        }

        /// Invariant: the outline stays within the samples' bounds grown by
        /// the stroke size, so a stroke never draws far from where it was made.
        #[test]
        fn prop_outline_stays_near_samples(
            raw in prop::collection::vec((-200f64..200., -200f64..200.), 2..40),
            size in 1f64..20.,
        ) {
            let samples: Vec<_> = raw.iter().map(|&(x, y)| Sample::new(x, y, 0.5)).collect();
            let o = outline(&samples, &Options { size, ..Options::default() });
            let (lx, hx) = raw.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.0), a.1.max(p.0)));
            let (ly, hy) = raw.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.1), a.1.max(p.1)));
            for p in o {
                prop_assert!(p[0] >= lx - size * 2.0 && p[0] <= hx + size * 2.0);
                prop_assert!(p[1] >= ly - size * 2.0 && p[1] <= hy + size * 2.0);
            }
        }
    }
}
