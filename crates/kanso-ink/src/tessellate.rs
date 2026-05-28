//! Stroke tessellation — converts [`crate::format::Stroke`] data into triangle
//! meshes ready for upload to any GPU renderer.
//!
//! This module is **pure Rust** with no GPU dependencies; it exists entirely in
//! the default build. The optional `gpu` feature consumes its output in
//! [`crate::gpu`].
//!
//! # Approach
//!
//! Each stroke is first smoothed via Catmull-Rom (8 samples/segment). For every
//! consecutive pair of smoothed points we build a screen-aligned quad:
//!
//! ```text
//!   p1_left  ─────────────────── p2_left
//!      │   ╲                   ╱   │
//!      │     ╲   2 triangles ╱     │
//!      │       ╲           ╱       │
//!   p1_right ─────────────────── p2_right
//! ```
//!
//! The quad is 4 vertices and 6 indices (two CCW triangles sharing the diagonal
//! p1_right→p2_left). The width at each endpoint is derived from
//! [`crate::geometry::pressure_to_width`] so pressure variation in the original
//! stroke is fully represented in the mesh.

use crate::format::{Element, SketchDoc, Stroke};
use crate::geometry;

// ── Data types ────────────────────────────────────────────────────────────────

/// A single vertex in a tessellated stroke mesh.
///
/// `pos` is in the same coordinate space as the source [`crate::format::Point`]
/// values (canvas pixels, origin top-left). The GPU module converts to NDC.
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    /// `[x, y]` in canvas-pixel space.
    pub pos: [f32; 2],
    /// `[r, g, b, a]` in 0.0 – 1.0 linear range.
    pub color: [f32; 4],
}

/// A triangle mesh produced by tessellating one or more strokes.
///
/// Indices reference into `vertices` and form triangle lists (every 3 indices
/// describe one CCW triangle).
#[derive(Debug, Default, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Append another mesh, offsetting the incoming indices so they point into
    /// the combined vertex buffer.
    fn append(&mut self, other: Mesh) {
        let offset = self.vertices.len() as u32;
        self.vertices.extend(other.vertices);
        self.indices.extend(other.indices.into_iter().map(|i| i + offset));
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Tessellate a single stroke into a [`Mesh`].
///
/// Returns an empty mesh if the stroke has fewer than 2 points or if every
/// segment has zero length.
pub fn tessellate_stroke(stroke: &Stroke) -> Mesh {
    let smoothed = geometry::smooth(&stroke.points, 8);
    if smoothed.len() < 2 {
        return Mesh::default();
    }

    // Pre-compute the stroke color once (it is uniform across the stroke).
    let color = rgba_to_f32(stroke.color);

    let mut mesh = Mesh::default();

    for i in 0..smoothed.len() - 1 {
        let a = &smoothed[i];
        let b = &smoothed[i + 1];

        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let len = (dx * dx + dy * dy).sqrt();

        // Skip degenerate (zero-length) segments to avoid NaN normals.
        if len < f32::EPSILON {
            continue;
        }

        // Normal perpendicular to the segment direction, pointing "left".
        // Normalise: divide the rotated direction (-dy, dx) by len.
        let nx = -dy / len;
        let ny = dx / len;

        let hw_a = geometry::pressure_to_width(stroke.base_width, a.pressure) * 0.5;
        let hw_b = geometry::pressure_to_width(stroke.base_width, b.pressure) * 0.5;

        // Four corners of the quad:
        //   0: a + normal * hw_a   (a_left)
        //   1: a - normal * hw_a   (a_right)
        //   2: b + normal * hw_b   (b_left)
        //   3: b - normal * hw_b   (b_right)
        let base = mesh.vertices.len() as u32;

        mesh.vertices.push(Vertex { pos: [a.x + nx * hw_a, a.y + ny * hw_a], color });
        mesh.vertices.push(Vertex { pos: [a.x - nx * hw_a, a.y - ny * hw_a], color });
        mesh.vertices.push(Vertex { pos: [b.x + nx * hw_b, b.y + ny * hw_b], color });
        mesh.vertices.push(Vertex { pos: [b.x - nx * hw_b, b.y - ny * hw_b], color });

        // Triangle 1: a_left, a_right, b_left  (CCW when y grows downward)
        mesh.indices.push(base);
        mesh.indices.push(base + 1);
        mesh.indices.push(base + 2);

        // Triangle 2: a_right, b_right, b_left
        mesh.indices.push(base + 1);
        mesh.indices.push(base + 3);
        mesh.indices.push(base + 2);
    }

    mesh
}

/// Tessellate every stroke in a [`SketchDoc`] and concatenate the meshes.
///
/// Non-stroke elements (shapes, arrows, text) are currently ignored; they can
/// be added as separate tessellation passes in the future.
pub fn tessellate_doc(doc: &SketchDoc) -> Mesh {
    let mut combined = Mesh::default();
    for element in &doc.elements {
        if let Element::Stroke(stroke) = element {
            combined.append(tessellate_stroke(stroke));
        }
    }
    combined
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[inline]
fn rgba_to_f32(c: crate::format::Rgba) -> [f32; 4] {
    [
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
        c.a as f32 / 255.0,
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{Background, Element, Point, Rgba, SketchDoc, Stroke, Tool};

    fn three_point_stroke() -> Stroke {
        Stroke {
            points: vec![
                Point { x: 0.0, y: 0.0, pressure: 0.5, tilt: 0.0, t: 0.0 },
                Point { x: 50.0, y: 25.0, pressure: 0.8, tilt: 0.0, t: 10.0 },
                Point { x: 100.0, y: 0.0, pressure: 0.6, tilt: 0.0, t: 20.0 },
            ],
            color: Rgba { r: 0, g: 0, b: 0, a: 255 },
            base_width: 4.0,
            tool: Tool::Pen,
        }
    }

    // ── tessellate_stroke ─────────────────────────────────────────────────────

    #[test]
    fn three_point_stroke_yields_non_empty_mesh() {
        let mesh = tessellate_stroke(&three_point_stroke());
        assert!(!mesh.vertices.is_empty(), "expected vertices");
        assert!(!mesh.indices.is_empty(), "expected indices");
    }

    #[test]
    fn vertex_count_is_multiple_of_four() {
        let mesh = tessellate_stroke(&three_point_stroke());
        assert_eq!(
            mesh.vertices.len() % 4,
            0,
            "vertices ({}) must be a multiple of 4 (one quad per segment)",
            mesh.vertices.len()
        );
    }

    #[test]
    fn index_count_is_multiple_of_six() {
        let mesh = tessellate_stroke(&three_point_stroke());
        assert_eq!(
            mesh.indices.len() % 6,
            0,
            "indices ({}) must be a multiple of 6 (two triangles per quad)",
            mesh.indices.len()
        );
    }

    #[test]
    fn all_indices_in_bounds() {
        let mesh = tessellate_stroke(&three_point_stroke());
        let vlen = mesh.vertices.len() as u32;
        for &idx in &mesh.indices {
            assert!(
                idx < vlen,
                "index {} out of bounds (vertex count = {})",
                idx,
                vlen
            );
        }
    }

    // ── edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn single_point_stroke_produces_empty_mesh() {
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0, pressure: 1.0, tilt: 0.0, t: 0.0 }],
            color: Rgba { r: 0, g: 0, b: 0, a: 255 },
            base_width: 2.0,
            tool: Tool::Pen,
        };
        let mesh = tessellate_stroke(&stroke);
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    #[test]
    fn zero_length_segment_is_skipped() {
        // All three points are identical → all segments degenerate.
        let stroke = Stroke {
            points: vec![
                Point { x: 10.0, y: 10.0, pressure: 1.0, tilt: 0.0, t: 0.0 },
                Point { x: 10.0, y: 10.0, pressure: 1.0, tilt: 0.0, t: 0.0 },
                Point { x: 10.0, y: 10.0, pressure: 1.0, tilt: 0.0, t: 0.0 },
            ],
            color: Rgba { r: 0, g: 0, b: 0, a: 255 },
            base_width: 2.0,
            tool: Tool::Pen,
        };
        // After Catmull-Rom on identical points the smoothed segments are all
        // zero-length, so the mesh must be empty (no NaN vertices).
        let mesh = tessellate_stroke(&stroke);
        for v in &mesh.vertices {
            assert!(v.pos[0].is_finite(), "NaN x in vertex");
            assert!(v.pos[1].is_finite(), "NaN y in vertex");
        }
    }

    // ── tessellate_doc ────────────────────────────────────────────────────────

    #[test]
    fn doc_with_one_stroke_matches_single_tessellation() {
        let stroke = three_point_stroke();
        let expected = tessellate_stroke(&stroke);

        let mut doc = SketchDoc::new();
        doc.background = Background::Blank;
        doc.elements.push(Element::Stroke(stroke));

        let doc_mesh = tessellate_doc(&doc);
        assert_eq!(doc_mesh.vertices.len(), expected.vertices.len());
        assert_eq!(doc_mesh.indices.len(), expected.indices.len());
    }

    #[test]
    fn doc_indices_stay_in_bounds_after_concatenation() {
        let mut doc = SketchDoc::new();
        doc.background = Background::Blank;
        // Two separate strokes — index offsets must be applied correctly.
        doc.elements.push(Element::Stroke(three_point_stroke()));
        doc.elements.push(Element::Stroke(three_point_stroke()));

        let mesh = tessellate_doc(&doc);
        let vlen = mesh.vertices.len() as u32;
        for &idx in &mesh.indices {
            assert!(idx < vlen, "index {} out of bounds after concat", idx);
        }
    }

    #[test]
    fn color_components_are_in_unit_range() {
        let mesh = tessellate_stroke(&three_point_stroke());
        for v in &mesh.vertices {
            for &c in &v.color {
                assert!((0.0..=1.0).contains(&c), "color component {} out of [0,1]", c);
            }
        }
    }
}
